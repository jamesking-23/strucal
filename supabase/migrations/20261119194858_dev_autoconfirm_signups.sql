-- DEVELOPMENT ONLY. Marks new accounts as email-confirmed so testing does not need email delivery.
-- Remove before real users sign up:  drop trigger autoconfirm_dev on auth.users;  drop function private.autoconfirm_dev();
create or replace function private.autoconfirm_dev() returns trigger language plpgsql security definer set search_path = '' as $$
begin
  new.email_confirmed_at := coalesce(new.email_confirmed_at, now());
  return new;
end $$;
revoke execute on function private.autoconfirm_dev() from public, anon, authenticated;
create trigger autoconfirm_dev before insert on auth.users for each row execute function private.autoconfirm_dev();
