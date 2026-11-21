create type public.org_role as enum ('owner','administrator','project_manager','designer','checker','approver','viewer','auditor','student');
create type public.rev_status as enum ('draft','submitted','changes_requested','checked','approved','rejected');

create table public.organisations (
  id uuid primary key default gen_random_uuid(),
  name text not null check (char_length(name) between 2 and 120),
  created_by uuid not null references auth.users(id),
  created_at timestamptz not null default now()
);
create table public.memberships (
  org_id uuid not null references public.organisations(id) on delete cascade,
  user_id uuid not null references auth.users(id) on delete cascade,
  role public.org_role not null,
  created_at timestamptz not null default now(),
  primary key (org_id, user_id)
);
create index memberships_user_idx on public.memberships(user_id);
create table public.profiles (
  id uuid primary key references auth.users(id) on delete cascade,
  display_name text not null default '',
  email text not null default '',
  created_at timestamptz not null default now()
);
create table public.projects (
  id uuid primary key default gen_random_uuid(),
  org_id uuid not null references public.organisations(id) on delete cascade,
  name text not null check (char_length(name) between 1 and 160),
  description text not null default '' check (char_length(description) <= 2000),
  created_by uuid not null references auth.users(id),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);
create index projects_org_idx on public.projects(org_id);
create table public.project_drafts (
  project_id uuid primary key references public.projects(id) on delete cascade,
  nodes jsonb not null default '[]'::jsonb check (jsonb_typeof(nodes) = 'array'),
  version integer not null default 1,
  updated_by uuid references auth.users(id),
  updated_at timestamptz not null default now()
);
create table public.revisions (
  id uuid primary key default gen_random_uuid(),
  project_id uuid not null references public.projects(id) on delete cascade,
  number integer not null,
  name text not null check (char_length(name) between 1 and 160),
  nodes jsonb not null check (jsonb_typeof(nodes) = 'array'),
  results jsonb not null,
  engine_version text not null default '',
  content_hash text not null,
  status public.rev_status not null default 'draft',
  created_by uuid not null references auth.users(id),
  created_at timestamptz not null default now(),
  unique (project_id, number)
);
create index revisions_project_idx on public.revisions(project_id);
create table public.reviews (
  id uuid primary key default gen_random_uuid(),
  revision_id uuid not null references public.revisions(id) on delete cascade,
  reviewer uuid not null references auth.users(id),
  kind text not null check (kind in ('comment','request_changes','recommend_approval','reject','approve')),
  comment text not null default '' check (char_length(comment) <= 4000),
  created_at timestamptz not null default now()
);
create index reviews_rev_idx on public.reviews(revision_id);
create table public.audit_events (
  id bigint generated always as identity primary key,
  org_id uuid not null references public.organisations(id) on delete cascade,
  actor uuid,
  action text not null,
  entity text not null,
  entity_id uuid,
  detail jsonb not null default '{}'::jsonb,
  created_at timestamptz not null default now(),
  prev_hash text not null,
  hash text not null
);
create index audit_org_idx on public.audit_events(org_id, id);

alter table public.organisations enable row level security;
alter table public.memberships enable row level security;
alter table public.profiles enable row level security;
alter table public.projects enable row level security;
alter table public.project_drafts enable row level security;
alter table public.revisions enable row level security;
alter table public.reviews enable row level security;
alter table public.audit_events enable row level security;

-- default Supabase grants are broad: remove them, then grant only what the policies need
revoke all on all tables in schema public from anon, authenticated;
grant select on public.organisations, public.memberships, public.profiles, public.projects, public.project_drafts, public.revisions, public.reviews, public.audit_events to authenticated;
grant update (name) on public.organisations to authenticated;
grant update (display_name) on public.profiles to authenticated;
grant update (name, description, updated_at) on public.projects to authenticated;
grant insert, update on public.project_drafts to authenticated;
