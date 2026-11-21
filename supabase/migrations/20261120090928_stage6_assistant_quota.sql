create table public.assistant_usage (
  user_id uuid not null references auth.users(id) on delete cascade,
  day date not null default (now() at time zone 'utc')::date,
  calls integer not null default 0,
  primary key (user_id, day)
);
alter table public.assistant_usage enable row level security;
revoke all on public.assistant_usage from anon, authenticated;

-- Takes one assistant call from the caller's daily allowance. Returns false when the allowance is used up.
create function public.assistant_take(p_limit integer default 40) returns boolean language plpgsql security definer set search_path = '' as $$
declare v_uid uuid := (select auth.uid()); v_calls integer;
begin
  if v_uid is null then raise exception 'Sign in required'; end if;
  insert into public.assistant_usage (user_id, day, calls) values (v_uid, (now() at time zone 'utc')::date, 1)
    on conflict (user_id, day) do update set calls = public.assistant_usage.calls + 1 returning calls into v_calls;
  if v_calls > greatest(1, least(p_limit, 500)) then
    update public.assistant_usage set calls = calls - 1 where user_id = v_uid and day = (now() at time zone 'utc')::date;
    return false;
  end if;
  return true;
end $$;
revoke execute on function public.assistant_take(integer) from public, anon;
grant execute on function public.assistant_take(integer) to authenticated;
