create schema if not exists private;
revoke all on schema private from public, anon;
grant usage on schema private to authenticated;

alter function public.is_member(uuid) set schema private;
alter function public.has_role(uuid, public.org_role[]) set schema private;
alter function public.project_org(uuid) set schema private;
alter function public.revision_org(uuid) set schema private;
alter function public.can_edit_project(uuid) set schema private;
alter function public.shares_org(uuid) set schema private;

-- project_org / revision_org only reveal an organisation to its own members
create or replace function private.project_org(p_project uuid) returns uuid language sql stable security definer set search_path = '' as $$
  select p.org_id from public.projects p where p.id = p_project
    and exists (select 1 from public.memberships m where m.org_id = p.org_id and m.user_id = (select auth.uid()));
$$;
create or replace function private.revision_org(p_rev uuid) returns uuid language sql stable security definer set search_path = '' as $$
  select p.org_id from public.revisions r join public.projects p on p.id = r.project_id
    where r.id = p_rev and exists (select 1 from public.memberships m where m.org_id = p.org_id and m.user_id = (select auth.uid()));
$$;

-- rewrite the RPCs so their bodies call the moved helpers
do $$
declare f record; def text;
begin
  for f in select p.oid from pg_proc p join pg_namespace n on n.oid = p.pronamespace where n.nspname = 'public'
           and p.proname in ('verify_audit_chain','create_project','save_revision','submit_revision','check_revision','approve_revision') loop
    def := pg_get_functiondef(f.oid);
    def := replace(def, 'public.has_role', 'private.has_role');
    def := replace(def, 'public.can_edit_project', 'private.can_edit_project');
    def := replace(def, 'public.project_org', 'private.project_org');
    def := replace(def, 'public.revision_org', 'private.revision_org');
    execute def;
  end loop;
end $$;
