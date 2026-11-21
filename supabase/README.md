# StruCal backend (Supabase)

Project: `strucal` (Frankfurt). Public client key is in `apps/web/lib/supabase.ts` and `apps/desktop/src/cloud.rs`. Never commit secret keys.

## Edge function
`functions/assistant/index.ts` - engineering assistant. Reads its model key from a Supabase secret:
`GEMINI_API_KEY` (preferred) or `ANTHROPIC_API_KEY`. Optional: `ASSISTANT_MODEL`, `ASSISTANT_DAILY_LIMIT` (default 40).

## Database migrations
Exported from the project's migration history into `migrations/` (same SQL, same order, version-prefixed). Apply in file order to a fresh Supabase project.

| File | Purpose |
|---|---|
| `20261119010422_stage5_tables.sql` | organisations, memberships, profiles, projects, drafts, revisions, reviews, audit events; row-level security enabled on every table; grants limited |
| `20261119010436_stage5_helpers_policies.sql` | role helpers, policies, sign-up profile trigger, immutability and append-only guards |
| `20261119085030_stage5_functions.sql` | audit hash chain, organisation/member/project functions, revision workflow (submit, check, approve) |
| `20261119085139_stage5_private_helpers.sql` | internal helpers moved to the `private` schema |
| `20261119194858_dev_autoconfirm_signups.sql` | **DEVELOPMENT ONLY** auto-confirm trigger. Remove before real users: `drop trigger autoconfirm_dev on auth.users; drop function private.autoconfirm_dev();` |
| `20261120011203_stage5_mfa_export_close.sql` | two-factor rule for review/approval, organisation export, account closure |
| `20261120090928_stage6_assistant_quota.sql` | daily assistant allowance |

Not captured here: Auth settings changed in the dashboard (email confirmation, MFA), and Edge Function secrets.
