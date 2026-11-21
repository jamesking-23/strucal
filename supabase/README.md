# StruCal backend (Supabase)

Project: `strucal` (Frankfurt). Public client key is in `apps/web/lib/supabase.ts` and `apps/desktop/src/cloud.rs`. Never commit secret keys.

## Edge function
`functions/assistant/index.ts` - engineering assistant. Reads its model key from a Supabase secret:
`GEMINI_API_KEY` (preferred) or `ANTHROPIC_API_KEY`. Optional: `ASSISTANT_MODEL`, `ASSISTANT_DAILY_LIMIT` (default 40).

## Database migrations applied (in order)
1. `stage5_tables` - organisations, memberships, profiles, projects, drafts, revisions, reviews, audit events; row-level security on every table.
2. `stage5_helpers_policies` - role helpers, policies, immutability and append-only guards.
3. `stage5_functions` - audit hash chain, organisation/member/project RPCs, revision workflow (submit, check, approve).
4. `stage5_private_helpers` - internal helpers moved to the `private` schema.
5. `dev_autoconfirm_signups` - DEVELOPMENT ONLY. Remove before real users: `drop trigger autoconfirm_dev on auth.users; drop function private.autoconfirm_dev();`
6. `stage5_mfa_export_close` - two-factor rule for review/approval, organisation export, account closure.
7. `stage6_assistant_quota` - daily assistant allowance.

The SQL for these migrations should be exported into `supabase/migrations/` (open item) so the schema is reproducible from the repository.
