alter table public.organisations add column require_mfa_approval boolean not null default true;

create function private.mfa_ok(p_org uuid) returns boolean language sql stable security definer set search_path = '' as $$
  select (not coalesce((select o.require_mfa_approval from public.organisations o where o.id = p_org), true))
      or coalesce((select auth.jwt() ->> 'aal'), 'aal1') = 'aal2';
$$;
revoke execute on function private.mfa_ok(uuid) from public, anon;
grant execute on function private.mfa_ok(uuid) to authenticated;

create function public.set_org_security(p_org uuid, p_require_mfa boolean) returns void language plpgsql security definer set search_path = '' as $$
begin
  if not private.has_role(p_org, array['owner','administrator']::public.org_role[]) then raise exception 'Only owners and administrators can change security settings'; end if;
  update public.organisations set require_mfa_approval = p_require_mfa where id = p_org;
  perform public.log_event(p_org, 'organisation.security_changed', 'organisation', p_org, jsonb_build_object('require_mfa_approval', p_require_mfa));
end $$;

-- enforce the two-factor rule inside the review and approval functions
do $$
declare def text; f record;
begin
  for f in select p.oid, p.proname from pg_proc p join pg_namespace n on n.oid = p.pronamespace where n.nspname = 'public' and p.proname in ('check_revision','approve_revision') loop
    def := pg_get_functiondef(f.oid);
    if f.proname = 'check_revision' then
      def := replace(def, E'if r.created_by = v_uid then raise exception ''You cannot check a revision you created''; end if;', E'if r.created_by = v_uid then raise exception ''You cannot check a revision you created''; end if;\n  if not private.mfa_ok(v_org) then raise exception ''Two-factor authentication required: verify your authenticator code, then try again''; end if;');
    else
      def := replace(def, E'if r.status <> ''checked'' then raise exception ''The revision must be checked before approval''; end if;', E'if r.status <> ''checked'' then raise exception ''The revision must be checked before approval''; end if;\n  if not private.mfa_ok(v_org) then raise exception ''Two-factor authentication required: verify your authenticator code, then try again''; end if;');
    end if;
    if position('private.mfa_ok' in def) = 0 then raise exception 'patch failed for %', f.proname; end if;
    execute def;
  end loop;
end $$;

create function public.export_organisation(p_org uuid) returns jsonb language plpgsql security definer set search_path = '' as $$
declare v jsonb;
begin
  if not private.has_role(p_org, array['owner','administrator','auditor']::public.org_role[]) then raise exception 'Only owners, administrators and auditors can export organisation data'; end if;
  select jsonb_build_object(
    'exported_at', now(),
    'organisation', (select to_jsonb(o) from public.organisations o where o.id = p_org),
    'members', (select coalesce(jsonb_agg(jsonb_build_object('user_id', m.user_id, 'role', m.role, 'email', pr.email, 'display_name', pr.display_name)), '[]'::jsonb) from public.memberships m left join public.profiles pr on pr.id = m.user_id where m.org_id = p_org),
    'projects', (select coalesce(jsonb_agg(to_jsonb(p)), '[]'::jsonb) from public.projects p where p.org_id = p_org),
    'drafts', (select coalesce(jsonb_agg(to_jsonb(d)), '[]'::jsonb) from public.project_drafts d join public.projects p on p.id = d.project_id where p.org_id = p_org),
    'revisions', (select coalesce(jsonb_agg(to_jsonb(r)), '[]'::jsonb) from public.revisions r join public.projects p on p.id = r.project_id where p.org_id = p_org),
    'reviews', (select coalesce(jsonb_agg(to_jsonb(rv)), '[]'::jsonb) from public.reviews rv join public.revisions r on r.id = rv.revision_id join public.projects p on p.id = r.project_id where p.org_id = p_org),
    'audit_events', (select coalesce(jsonb_agg(to_jsonb(a) order by a.id), '[]'::jsonb) from public.audit_events a where a.org_id = p_org)
  ) into v;
  perform public.log_event(p_org, 'organisation.exported', 'organisation', p_org, '{}'::jsonb);
  return v;
end $$;

create function public.close_account() returns void language plpgsql security definer set search_path = '' as $$
declare v_uid uuid := (select auth.uid()); m record;
begin
  if v_uid is null then raise exception 'Sign in required'; end if;
  if exists (select 1 from public.memberships x where x.user_id = v_uid and x.role = 'owner' and (select count(*) from public.memberships y where y.org_id = x.org_id and y.role = 'owner') = 1) then
    raise exception 'You are the only owner of an organisation. Make someone else an owner before closing your account';
  end if;
  for m in select org_id, role from public.memberships where user_id = v_uid loop
    perform public.log_event(m.org_id, 'member.removed', 'membership', v_uid, jsonb_build_object('role', m.role, 'reason', 'account_closed'));
  end loop;
  delete from public.memberships where user_id = v_uid;
  update public.profiles set display_name = 'Closed account', email = '' where id = v_uid;
  update auth.users set banned_until = 'infinity'::timestamptz where id = v_uid;
end $$;

revoke execute on function public.set_org_security(uuid, boolean), public.export_organisation(uuid), public.close_account() from public, anon;
grant execute on function public.set_org_security(uuid, boolean), public.export_organisation(uuid), public.close_account() to authenticated;
