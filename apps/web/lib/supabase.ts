import { createClient } from "@supabase/supabase-js";

// Public values: the publishable key is designed to ship in client code. Access is enforced by row-level security in the database.
export const SUPABASE_URL = "https://bunshysrzoozplnvxhui.supabase.co";
export const SUPABASE_KEY = "sb_publishable_Z1waCuLYUVRaGV9LyWxLAw_gbl-HLb9";
export const supabase = createClient(SUPABASE_URL, SUPABASE_KEY, { auth: { persistSession: true, autoRefreshToken: true } });

export const ROLES = ["owner", "administrator", "project_manager", "designer", "checker", "approver", "viewer", "auditor", "student"] as const;
export const ROLE_LABEL: Record<string, string> = { owner: "Owner", administrator: "Administrator", project_manager: "Project manager", designer: "Designer", checker: "Checker", approver: "Approver", viewer: "Viewer", auditor: "Auditor", student: "Student" };
export const STATUS_LABEL: Record<string, string> = { draft: "Draft", submitted: "Submitted", changes_requested: "Changes requested", checked: "Checked: recommended for approval", approved: "Approved", rejected: "Rejected" };
