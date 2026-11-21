create function public.log_event(p_org uuid, p_action text, p_entity text, p_entity_id uuid, p_detail jsonb) returns void language plpgsql security definer set search_path = '' as $$
declare v_prev text; v_ts timestamptz := clock_timestamp(); v_hash text; v_actor uuid := (select auth.uid());
begin
  perform pg_advisory_xact_lock(hashtext(p_org::text));
  select hash into v_prev from public.audit_events where org_id = p_org order by id desc limit 1;
  v_prev := coalesce(v_prev, 'GENESIS');
  v_hash := encode(sha256(convert_to(v_prev || '|' || p_org::text || '|' || coalesce(v_actor::text, '') || '|' || p_action || '|' || p_entity || '|' || coalesce(p_entity_id::text, '') || '|' || p_detail::text || '|' || to_char(v_ts at time zone 'utc', 'YYYY-MM-DD"T"HH24:MI:SS.US'), 'UTF8')), 'hex');
  insert into public.audit_events (org_id, actor, action, entity, entity_id, detail, created_at, prev_hash, hash) values (p_org, v_actor, p_action, p_entity, p_entity_id, p_detail, v_ts, v_prev, v_hash);
end $$;
revoke execute on function public.log_event(uuid, text, text, uuid, jsonb) from public, anon, authenticated;

create function public.verify_audit_chain(p_org uuid) returns table (ok boolean, checked bigint, broken_at bigint) language plpgsql security definer set search_path = '' as $$
declare r record; v_prev text := 'GENESIS'; v_n bigint := 0; v_calc text;
begin
  if not public.has_role(p_org, array['owner','administrator','auditor']::public.org_role[]) then raise exception 'Not permitted'; end if;
  for r in select * from public.audit_events where org_id = p_org order by id loop
    v_calc := encode(sha256(convert_to(v_prev || '|' || r.org_id::text || '|' || coalesce(r.actor::text, '') || '|' || r.action || '|' || r.entity || '|' || coalesce(r.entity_id::text, '') || '|' || r.detail::text || '|' || to_char(r.created_at at time zone 'utc', 'YYYY-MM-DD"T"HH24:MI:SS.US'), 'UTF8')), 'hex');
    if r.prev_hash <> v_prev or r.hash <> v_calc then return query select false, v_n, r.id; return; end if;
    v_prev := r.hash; v_n := v_n + 1;
  end loop;
  return query select true, v_n, null::bigint;
end $$;

create function public.create_organisation(p_name text) returns uuid language plpgsql security definer set search_path = '' as $$
declare v_id uuid; v_uid uuid := (select auth.uid()); v_name text := btrim(p_name);
begin
  if v_uid is null then raise exception 'Sign in required'; end if;
  if char_length(v_name) < 2 or char_length(v_name) > 120 then raise exception 'Organisation name must be 2-120 characters'; end if;
  if (select count(*) from public.organisations where created_by = v_uid) >= 10 then raise exception 'Organisation limit reached'; end if;
  insert into public.organisations (name, created_by) values (v_name, v_uid) returning id into v_id;
  insert into public.memberships (org_id, user_id, role) values (v_id, v_uid, 'owner');
  perform public.log_event(v_id, 'organisation.created', 'organisation', v_id, jsonb_build_object('name', v_name));
  return v_id;
end $$;

create function public.add_member(p_org uuid, p_email text, p_role public.org_role) returns void language plpgsql security definer set search_path = '' as $$
declare v_caller public.org_role; v_target uuid;
begin
  select role into v_caller from public.memberships where org_id = p_org and user_id = (select auth.uid());
  if v_caller is null or v_caller not in ('owner','administrator') then raise exception 'Only owners and administrators can add members'; end if;
  if p_role = 'owner' and v_caller <> 'owner' then raise exception 'Only an owner can grant the owner role'; end if;
  select id into v_target from public.profiles where email = lower(btrim(p_email));
  if v_target is null then raise exception 'No registered user with that email. Ask them to sign up first.'; end if;
  if exists (select 1 from public.memberships where org_id = p_org and user_id = v_target) then raise exception 'Already a member'; end if;
  insert into public.memberships (org_id, user_id, role) values (p_org, v_target, p_role);
  perform public.log_event(p_org, 'member.added', 'membership', v_target, jsonb_build_object('role', p_role));
end $$;

create function public.set_member_role(p_org uuid, p_user uuid, p_role public.org_role) returns void language plpgsql security definer set search_path = '' as $$
declare v_caller public.org_role; v_old public.org_role;
begin
  select role into v_caller from public.memberships where org_id = p_org and user_id = (select auth.uid());
  if v_caller is null or v_caller not in ('owner','administrator') then raise exception 'Only owners and administrators can change roles'; end if;
  select role into v_old from public.memberships where org_id = p_org and user_id = p_user for update;
  if v_old is null then raise exception 'Not a member'; end if;
  if (p_role = 'owner' or v_old = 'owner') and v_caller <> 'owner' then raise exception 'Only an owner can grant or change the owner role'; end if;
  if v_old = 'owner' and p_role <> 'owner' and (select count(*) from public.memberships where org_id = p_org and role = 'owner') <= 1 then raise exception 'An organisation must keep at least one owner'; end if;
  update public.memberships set role = p_role where org_id = p_org and user_id = p_user;
  perform public.log_event(p_org, 'member.role_changed', 'membership', p_user, jsonb_build_object('from', v_old, 'to', p_role));
end $$;

create function public.remove_member(p_org uuid, p_user uuid) returns void language plpgsql security definer set search_path = '' as $$
declare v_caller public.org_role; v_old public.org_role; v_uid uuid := (select auth.uid());
begin
  select role into v_caller from public.memberships where org_id = p_org and user_id = v_uid;
  if v_caller is null then raise exception 'Not permitted'; end if;
  if p_user <> v_uid and v_caller not in ('owner','administrator') then raise exception 'Only owners and administrators can remove members'; end if;
  select role into v_old from public.memberships where org_id = p_org and user_id = p_user for update;
  if v_old is null then raise exception 'Not a member'; end if;
  if v_old = 'owner' and v_caller <> 'owner' then raise exception 'Only an owner can remove an owner'; end if;
  if v_old = 'owner' and (select count(*) from public.memberships where org_id = p_org and role = 'owner') <= 1 then raise exception 'An organisation must keep at least one owner'; end if;
  delete from public.memberships where org_id = p_org and user_id = p_user;
  perform public.log_event(p_org, 'member.removed', 'membership', p_user, jsonb_build_object('role', v_old));
end $$;

create function public.create_project(p_org uuid, p_name text, p_description text default '') returns uuid language plpgsql security definer set search_path = '' as $$
declare v_id uuid; v_uid uuid := (select auth.uid());
begin
  if not public.has_role(p_org, array['owner','administrator','project_manager','designer','student']::public.org_role[]) then raise exception 'Your role cannot create projects'; end if;
  if char_length(btrim(p_name)) < 1 or char_length(p_name) > 160 then raise exception 'Project name must be 1-160 characters'; end if;
  insert into public.projects (org_id, name, description, created_by) values (p_org, btrim(p_name), coalesce(p_description, ''), v_uid) returning id into v_id;
  insert into public.project_drafts (project_id, updated_by) values (v_id, v_uid);
  perform public.log_event(p_org, 'project.created', 'project', v_id, jsonb_build_object('name', btrim(p_name)));
  return v_id;
end $$;

create function public.save_revision(p_project uuid, p_name text, p_nodes jsonb, p_results jsonb, p_engine text default '') returns uuid language plpgsql security definer set search_path = '' as $$
declare v_id uuid; v_n integer; v_org uuid; v_hash text;
begin
  if not public.can_edit_project(p_project) then raise exception 'You cannot save revisions in this project'; end if;
  if jsonb_typeof(p_nodes) <> 'array' then raise exception 'Invalid project data'; end if;
  if octet_length(p_nodes::text) > 2000000 or octet_length(p_results::text) > 2000000 then raise exception 'Project data is too large'; end if;
  select org_id into v_org from public.projects where id = p_project;
  perform pg_advisory_xact_lock(hashtext(p_project::text));
  select coalesce(max(number), 0) + 1 into v_n from public.revisions where project_id = p_project;
  v_hash := encode(sha256(convert_to(p_nodes::text || '|' || p_results::text, 'UTF8')), 'hex');
  insert into public.revisions (project_id, number, name, nodes, results, engine_version, content_hash, created_by) values (p_project, v_n, coalesce(nullif(btrim(p_name), ''), 'Rev ' || v_n), p_nodes, p_results, coalesce(p_engine, ''), v_hash, (select auth.uid())) returning id into v_id;
  perform public.log_event(v_org, 'revision.saved', 'revision', v_id, jsonb_build_object('project', p_project, 'number', v_n, 'hash', v_hash));
  return v_id;
end $$;

create function public.submit_revision(p_rev uuid) returns void language plpgsql security definer set search_path = '' as $$
declare r public.revisions; v_org uuid;
begin
  select * into r from public.revisions where id = p_rev for update;
  if r.id is null then raise exception 'Revision not found'; end if;
  if not public.can_edit_project(r.project_id) then raise exception 'You cannot submit this revision'; end if;
  if r.status not in ('draft','changes_requested') then raise exception 'Only a draft revision, or one with changes requested, can be submitted'; end if;
  update public.revisions set status = 'submitted' where id = p_rev;
  v_org := public.project_org(r.project_id);
  perform public.log_event(v_org, 'revision.submitted', 'revision', p_rev, jsonb_build_object('number', r.number));
end $$;

create function public.check_revision(p_rev uuid, p_outcome text, p_comment text default '') returns void language plpgsql security definer set search_path = '' as $$
declare r public.revisions; v_org uuid; v_uid uuid := (select auth.uid()); v_new public.rev_status; v_kind text;
begin
  select * into r from public.revisions where id = p_rev for update;
  if r.id is null then raise exception 'Revision not found'; end if;
  v_org := public.project_org(r.project_id);
  if not public.has_role(v_org, array['checker']::public.org_role[]) then raise exception 'Only a checker can check a revision'; end if;
  if r.created_by = v_uid then raise exception 'You cannot check a revision you created'; end if;
  if r.status not in ('submitted','checked') then raise exception 'This revision is not awaiting checking'; end if;
  if p_outcome not in ('comment','recommend_approval','request_changes','reject') then raise exception 'Unknown outcome'; end if;
  if p_outcome in ('request_changes','reject') and btrim(coalesce(p_comment, '')) = '' then raise exception 'A comment is required'; end if;
  v_kind := p_outcome;
  v_new := case p_outcome when 'recommend_approval' then 'checked' when 'request_changes' then 'changes_requested' when 'reject' then 'rejected' else r.status end;
  insert into public.reviews (revision_id, reviewer, kind, comment) values (p_rev, v_uid, v_kind, coalesce(p_comment, ''));
  if v_new <> r.status then update public.revisions set status = v_new where id = p_rev; end if;
  perform public.log_event(v_org, 'revision.checked', 'revision', p_rev, jsonb_build_object('outcome', p_outcome, 'status', v_new));
end $$;

create function public.approve_revision(p_rev uuid, p_approve boolean, p_comment text default '') returns void language plpgsql security definer set search_path = '' as $$
declare r public.revisions; v_org uuid; v_uid uuid := (select auth.uid()); v_checker uuid;
begin
  select * into r from public.revisions where id = p_rev for update;
  if r.id is null then raise exception 'Revision not found'; end if;
  v_org := public.project_org(r.project_id);
  if not public.has_role(v_org, array['approver']::public.org_role[]) then raise exception 'Only an approver can approve or reject a checked revision'; end if;
  if r.status <> 'checked' then raise exception 'The revision must be checked before approval'; end if;
  if r.created_by = v_uid then raise exception 'You cannot approve a revision you created'; end if;
  select reviewer into v_checker from public.reviews where revision_id = p_rev and kind = 'recommend_approval' order by created_at desc limit 1;
  if v_checker = v_uid then raise exception 'The approver must be a different person from the checker'; end if;
  if not p_approve and btrim(coalesce(p_comment, '')) = '' then raise exception 'A comment is required to reject'; end if;
  insert into public.reviews (revision_id, reviewer, kind, comment) values (p_rev, v_uid, case when p_approve then 'approve' else 'reject' end, coalesce(p_comment, ''));
  update public.revisions set status = case when p_approve then 'approved'::public.rev_status else 'rejected'::public.rev_status end where id = p_rev;
  perform public.log_event(v_org, case when p_approve then 'revision.approved' else 'revision.rejected' end, 'revision', p_rev, jsonb_build_object('number', r.number, 'hash', r.content_hash));
end $$;

revoke execute on function public.verify_audit_chain(uuid), public.create_organisation(text), public.add_member(uuid, text, public.org_role), public.set_member_role(uuid, uuid, public.org_role), public.remove_member(uuid, uuid), public.create_project(uuid, text, text), public.save_revision(uuid, text, jsonb, jsonb, text), public.submit_revision(uuid), public.check_revision(uuid, text, text), public.approve_revision(uuid, boolean, text) from public, anon;
grant execute on function public.verify_audit_chain(uuid), public.create_organisation(text), public.add_member(uuid, text, public.org_role), public.set_member_role(uuid, uuid, public.org_role), public.remove_member(uuid, uuid), public.create_project(uuid, text, text), public.save_revision(uuid, text, jsonb, jsonb, text), public.submit_revision(uuid), public.check_revision(uuid, text, text), public.approve_revision(uuid, boolean, text) to authenticated;
