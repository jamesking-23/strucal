-- helpers (security definer so policies do not recurse into RLS)
create function public.is_member(p_org uuid) returns boolean language sql stable security definer set search_path = '' as $$
  select exists (select 1 from public.memberships where org_id = p_org and user_id = (select auth.uid()));
$$;
create function public.has_role(p_org uuid, p_roles public.org_role[]) returns boolean language sql stable security definer set search_path = '' as $$
  select exists (select 1 from public.memberships where org_id = p_org and user_id = (select auth.uid()) and role = any (p_roles));
$$;
create function public.project_org(p_project uuid) returns uuid language sql stable security definer set search_path = '' as $$
  select org_id from public.projects where id = p_project;
$$;
create function public.revision_org(p_rev uuid) returns uuid language sql stable security definer set search_path = '' as $$
  select p.org_id from public.revisions r join public.projects p on p.id = r.project_id where r.id = p_rev;
$$;
create function public.can_edit_project(p_project uuid) returns boolean language sql stable security definer set search_path = '' as $$
  select exists (
    select 1 from public.projects p join public.memberships m on m.org_id = p.org_id and m.user_id = (select auth.uid())
    where p.id = p_project and (m.role in ('owner','administrator','project_manager','designer') or (m.role = 'student' and p.created_by = (select auth.uid())))
  );
$$;
create function public.shares_org(p_user uuid) returns boolean language sql stable security definer set search_path = '' as $$
  select exists (select 1 from public.memberships a join public.memberships b on a.org_id = b.org_id where a.user_id = (select auth.uid()) and b.user_id = p_user);
$$;
revoke execute on function public.is_member(uuid), public.has_role(uuid, public.org_role[]), public.project_org(uuid), public.revision_org(uuid), public.can_edit_project(uuid), public.shares_org(uuid) from public, anon;
grant execute on function public.is_member(uuid), public.has_role(uuid, public.org_role[]), public.project_org(uuid), public.revision_org(uuid), public.can_edit_project(uuid), public.shares_org(uuid) to authenticated;

-- policies
create policy org_select on public.organisations for select to authenticated using (public.is_member(id));
create policy org_update on public.organisations for update to authenticated using (public.has_role(id, array['owner','administrator']::public.org_role[])) with check (public.has_role(id, array['owner','administrator']::public.org_role[]));
create policy mem_select on public.memberships for select to authenticated using (public.is_member(org_id));
create policy prof_select on public.profiles for select to authenticated using (id = (select auth.uid()) or public.shares_org(id));
create policy prof_update on public.profiles for update to authenticated using (id = (select auth.uid())) with check (id = (select auth.uid()));
create policy proj_select on public.projects for select to authenticated using (public.is_member(org_id));
create policy proj_update on public.projects for update to authenticated using (public.has_role(org_id, array['owner','administrator','project_manager']::public.org_role[])) with check (public.has_role(org_id, array['owner','administrator','project_manager']::public.org_role[]));
create policy draft_select on public.project_drafts for select to authenticated using (public.is_member(public.project_org(project_id)));
create policy draft_insert on public.project_drafts for insert to authenticated with check (public.can_edit_project(project_id) and updated_by = (select auth.uid()));
create policy draft_update on public.project_drafts for update to authenticated using (public.can_edit_project(project_id)) with check (public.can_edit_project(project_id) and updated_by = (select auth.uid()));
create policy rev_select on public.revisions for select to authenticated using (public.is_member(public.project_org(project_id)));
create policy review_select on public.reviews for select to authenticated using (public.is_member(public.revision_org(revision_id)));
create policy audit_select on public.audit_events for select to authenticated using (public.has_role(org_id, array['owner','administrator','auditor']::public.org_role[]));

-- profile row on sign-up
create function public.handle_new_user() returns trigger language plpgsql security definer set search_path = '' as $$
begin
  insert into public.profiles (id, display_name, email) values (new.id, coalesce(nullif(new.raw_user_meta_data ->> 'display_name', ''), split_part(new.email, '@', 1)), lower(coalesce(new.email, '')));
  return new;
end $$;
revoke execute on function public.handle_new_user() from public, anon, authenticated;
create trigger on_auth_user_created after insert on auth.users for each row execute function public.handle_new_user();

-- immutability: revisions content can never change; audit rows can never be edited or removed
create function public.guard_revision() returns trigger language plpgsql set search_path = '' as $$
begin
  if tg_op = 'DELETE' then raise exception 'Revisions cannot be deleted'; end if;
  if new.nodes is distinct from old.nodes or new.results is distinct from old.results or new.content_hash <> old.content_hash or new.number <> old.number or new.project_id <> old.project_id or new.created_by <> old.created_by or new.created_at <> old.created_at then
    raise exception 'Revision content is immutable';
  end if;
  if old.status in ('approved','rejected') and new.status <> old.status then raise exception 'A % revision is final', old.status; end if;
  return new;
end $$;
create trigger revisions_guard before update or delete on public.revisions for each row execute function public.guard_revision();
create function public.guard_audit() returns trigger language plpgsql set search_path = '' as $$
begin raise exception 'Audit events are append-only'; end $$;
create trigger audit_guard before update or delete on public.audit_events for each row execute function public.guard_audit();
create function public.guard_reviews() returns trigger language plpgsql set search_path = '' as $$
begin raise exception 'Reviews are append-only'; end $$;
create trigger reviews_guard before update or delete on public.reviews for each row execute function public.guard_reviews();
