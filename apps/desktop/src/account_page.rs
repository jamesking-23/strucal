//! Account and teams: sign-in, organisations, roles, projects, check and approval workflow, audit trail (same backend as the web app).
use super::*;
use crate::cloud::Client;
use serde_json::{json, Value};
use std::sync::mpsc::{channel, Receiver};

const ROLES: [(&str, &str); 9] = [
    ("owner", "Owner"),
    ("administrator", "Administrator"),
    ("project_manager", "Project manager"),
    ("designer", "Designer"),
    ("checker", "Checker"),
    ("approver", "Approver"),
    ("viewer", "Viewer"),
    ("auditor", "Auditor"),
    ("student", "Student"),
];
fn role_label(r: &str) -> &str {
    ROLES.iter().find(|x| x.0 == r).map(|x| x.1).unwrap_or(r)
}
fn status_label(s: &str) -> &str {
    match s {
        "draft" => "Draft",
        "submitted" => "Submitted",
        "changes_requested" => "Changes requested",
        "checked" => "Checked: recommended for approval",
        "approved" => "Approved",
        "rejected" => "Rejected",
        o => o,
    }
}

#[derive(Clone)]
enum Act {
    LoadAll,
    LoadRevs,
    StepUp,
    SetMfaRule(bool),
    Export,
    CloseAccount,
    SignIn,
    SignUp,
    SignOut,
    CreateOrg,
    CreateProj,
    AddMember,
    SetRole(String, String),
    Remove(String),
    Submit(String),
    Check(String, &'static str),
    Approve(String, bool),
    Verify,
    MfaEnroll,
    MfaVerify,
    Reload,
}

/// Network work runs on a worker thread with a copy of the page state and client; both are swapped back on completion.
type Done = (Acct, Client);

#[derive(Default, Clone)]
pub struct Acct {
    email: String,
    pw: String,
    name: String,
    signup: bool,
    msg: String,
    err: String,
    orgs: Vec<(String, String, String, bool)>,
    org: usize,
    members: Vec<Value>,
    projects: Vec<Value>,
    proj: usize,
    revs: Vec<Value>,
    reviews: Vec<Value>,
    audit: Vec<Value>,
    chain: String,
    new_org: String,
    new_proj: String,
    add_email: String,
    add_role: usize,
    comment: String,
    mfa_factor: String,
    mfa_secret: String,
    mfa_code: String,
    loaded_for: String,
    step_code: String,
    close_ok: bool,
    /// Destructive action waiting for confirmation: (question, action).
    confirm: Option<(String, Act)>,
}

/// Holds the receiver of a running account job (not cloned with the page state).
#[derive(Default)]
pub struct AcctJob {
    rx: Option<Receiver<Done>>,
}

fn s(v: &Value, k: &str) -> String {
    v[k].as_str().unwrap_or("").to_string()
}

impl Acct {
    pub fn new() -> Self {
        Acct { add_role: 3, ..Default::default() }
    }
    fn role(&self) -> String {
        self.orgs.get(self.org).map(|o| o.2.clone()).unwrap_or_default()
    }
    fn is_admin(&self) -> bool {
        matches!(self.role().as_str(), "owner" | "administrator")
    }
    fn who(&self, id: &str) -> String {
        self.members
            .iter()
            .find(|m| s(m, "user_id") == id)
            .map(|m| {
                let n = s(m, "display_name");
                if n.is_empty() {
                    s(m, "email")
                } else {
                    n
                }
            })
            .unwrap_or("?".into())
    }
    fn ok<T>(&mut self, r: Result<T, String>, done: &str) -> Option<T> {
        match r {
            Ok(v) => {
                self.err.clear();
                self.msg = done.into();
                Some(v)
            }
            Err(e) => {
                self.msg.clear();
                self.err = e;
                None
            }
        }
    }

    fn load_orgs(&mut self, cl: &mut Client) {
        let r = cl.get(&format!("memberships?select=role,organisations(id,name,require_mfa_approval)&user_id=eq.{}", cl.user_id()));
        if let Some(v) = self.ok(r, "") {
            self.orgs = v
                .as_array()
                .cloned()
                .unwrap_or_default()
                .iter()
                .map(|m| {
                    (
                        s(&m["organisations"], "id"),
                        s(&m["organisations"], "name"),
                        s(m, "role"),
                        m["organisations"]["require_mfa_approval"].as_bool().unwrap_or(true),
                    )
                })
                .collect();
            if self.org >= self.orgs.len() {
                self.org = 0;
            }
        }
        self.load_org(cl);
    }
    fn load_org(&mut self, cl: &mut Client) {
        self.members.clear();
        self.projects.clear();
        self.audit.clear();
        self.revs.clear();
        self.reviews.clear();
        let Some((org, _, _, _)) = self.orgs.get(self.org).cloned() else { return };
        if let Ok(ms) = cl.get(&format!("memberships?select=user_id,role&org_id=eq.{org}")) {
            let list = ms.as_array().cloned().unwrap_or_default();
            let ids: Vec<String> = list.iter().map(|m| s(m, "user_id")).collect();
            let profiles = cl.get(&format!("profiles?select=id,display_name,email&id=in.({})", ids.join(","))).unwrap_or(Value::Null);
            self.members = list
                .iter()
                .map(|m| {
                    let p =
                        profiles.as_array().and_then(|a| a.iter().find(|x| s(x, "id") == s(m, "user_id"))).cloned().unwrap_or(Value::Null);
                    json!({"user_id": m["user_id"], "role": m["role"], "display_name": p["display_name"], "email": p["email"]})
                })
                .collect();
        }
        if let Ok(p) = cl.get(&format!("projects?select=id,name,description&org_id=eq.{org}&order=created_at.desc")) {
            self.projects = p.as_array().cloned().unwrap_or_default();
        }
        if self.proj >= self.projects.len() {
            self.proj = 0;
        }
        if matches!(self.role().as_str(), "owner" | "administrator" | "auditor") {
            if let Ok(a) = cl.get(&format!("audit_events?select=id,actor,action,detail,created_at&org_id=eq.{org}&order=id.desc&limit=40"))
            {
                self.audit = a.as_array().cloned().unwrap_or_default();
            }
        }
        self.load_revs(cl);
    }
    fn load_revs(&mut self, cl: &mut Client) {
        self.revs.clear();
        self.reviews.clear();
        let Some(p) = self.projects.get(self.proj) else { return };
        let pid = s(p, "id");
        if let Ok(r) = cl.get(&format!(
            "revisions?select=id,number,name,status,created_by,created_at,content_hash&project_id=eq.{pid}&order=number.desc"
        )) {
            self.revs = r.as_array().cloned().unwrap_or_default();
            let ids: Vec<String> = self.revs.iter().map(|x| s(x, "id")).collect();
            if !ids.is_empty() {
                if let Ok(rv) = cl.get(&format!(
                    "reviews?select=id,revision_id,reviewer,kind,comment,created_at&revision_id=in.({})&order=created_at",
                    ids.join(",")
                )) {
                    self.reviews = rv.as_array().cloned().unwrap_or_default();
                }
            }
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context, cl: &mut Client, job: &mut AcctJob) -> Option<(String, String)> {
        if let Some(rx) = &job.rx {
            match rx.try_recv() {
                Ok((a, c)) => {
                    *self = a;
                    *cl = c;
                    job.rx = None;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    job.rx = None;
                    self.err = "The request stopped unexpectedly. Try again.".into();
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => ctx.request_repaint_after(std::time::Duration::from_millis(100)),
            }
        }
        let busy = job.rx.is_some();
        let mut acts: Vec<Act> = vec![];
        let mut open = None;
        if !busy && cl.signed_in() && self.loaded_for != cl.user_id() {
            self.loaded_for = cl.user_id().clone();
            acts.push(Act::LoadAll);
        }
        if !cl.signed_in() {
            self.loaded_for.clear();
        }
        if let Some((q, a)) = self.confirm.clone() {
            egui::Window::new("Please confirm").collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(
                ctx,
                |ui| {
                    ui.label(q);
                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new(RichText::new("Confirm").color(on_ink())).fill(red())).clicked() {
                            acts.push(a.clone());
                            self.confirm = None;
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm = None;
                        }
                    });
                },
            );
        }
        egui::CentralPanel::default().frame(egui::Frame::new().fill(bg()).inner_margin(egui::Margin::same(10))).show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.set_max_width(1100.0);
                if busy { ui.horizontal(|ui| { ui.spinner(); ui.label(RichText::new("Working…").color(mute())); }); ui.disable(); }
                if self.confirm.is_some() { ui.disable(); }
                if !self.err.is_empty() { ui.colored_label(red(), &self.err); }
                if !self.msg.is_empty() { ui.colored_label(green(), &self.msg); }
                if !cl.signed_in() {
                    ui.add_space(40.0);
                    ui.horizontal_top(|ui| {
                        ui.add_space(((ui.available_width() - 980.0) / 2.0).max(0.0));
                        // copy column
                        ui.vertical(|ui| {
                            ui.set_width(520.0);
                            ui.label(RichText::new("STRUCAL ACCOUNT").font(strong(10.5)).extra_letter_spacing(1.2).color(mute()));
                            ui.add_space(6.0);
                            ui.label(RichText::new("Sign in to share projects and sign off revisions.").font(strong(30.0)).color(tx()));
                            ui.add_space(14.0);
                            let (hr, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 2.0), egui::Sense::hover());
                            ui.painter().rect_filled(hr, 0.0, tx());
                            for (t, d) in [
                                ("Cloud projects", "Linked calculations saved with optimistic locking."),
                                ("Check and approval", "Designer, checker and approver are three different people."),
                                ("Audit trail", "Every action recorded in a tamper-evident chain."),
                                ("Stays signed in", "Your session is kept securely and restored when StruCal starts."),
                            ] {
                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    ui.allocate_ui_with_layout(egui::vec2(170.0, 18.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                        ui.set_width(170.0);
                                        ui.label(RichText::new(t).font(strong(13.0)));
                                    });
                                    ui.label(RichText::new(d).color(mute()));
                                });
                                ui.add_space(8.0);
                                rule(ui);
                            }
                            ui.add_space(10.0);
                            ui.label(RichText::new("The workbench works without an account; inputs stay on this computer.").size(12.0).color(p().ink4));
                        });
                        ui.add_space(48.0);
                        // form column
                        ui.vertical(|ui| {
                            ui.set_width(400.0);
                            let r = egui::Frame::new().fill(surf()).stroke(egui::Stroke::new(1.0_f32, p().line_strong)).inner_margin(egui::Margin::same(20)).show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.spacing_mut().item_spacing.y = 8.0;
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 0.0;
                                    let w = ui.available_width() / 2.0;
                                    for (up, l) in [(false, "Sign in"), (true, "Create account")] {
                                        let on = self.signup == up;
                                        let b = egui::Button::new(RichText::new(l).font(medium(12.5)).color(if on { on_ink() } else { tx() }))
                                            .fill(if on { tx() } else { surf() })
                                            .stroke(egui::Stroke::new(1.0_f32, if on { tx() } else { p().line_strong }))
                                            .corner_radius(egui::CornerRadius::ZERO)
                                            .min_size(egui::vec2(w, 30.0));
                                        if ui.add(b).clicked() && !on {
                                            self.signup = up;
                                            self.err.clear();
                                        }
                                    }
                                });
                                let field = |ui: &mut egui::Ui, l: &str| ui.label(RichText::new(l.to_uppercase()).font(strong(10.5)).extra_letter_spacing(0.7).color(mute()));
                                if self.signup {
                                    field(ui, "Display name · shown on reviews and approvals");
                                    ui.add(egui::TextEdit::singleline(&mut self.name).desired_width(f32::INFINITY).margin(egui::Margin::symmetric(8, 6)));
                                }
                                field(ui, "Email");
                                ui.add(egui::TextEdit::singleline(&mut self.email).desired_width(f32::INFINITY).margin(egui::Margin::symmetric(8, 6)));
                                field(ui, "Password · at least 8 characters");
                                let pw = ui.add(egui::TextEdit::singleline(&mut self.pw).password(true).desired_width(f32::INFINITY).margin(egui::Margin::symmetric(8, 6)));
                                ui.add_space(4.0);
                                let label = if self.signup { "Create account" } else { "Sign in" };
                                let b = egui::Button::new(RichText::new(label).font(strong(13.0)).color(on_ink())).fill(tx()).corner_radius(egui::CornerRadius::ZERO).min_size(egui::vec2(ui.available_width(), 36.0));
                                let go = ui.add(b).clicked() || (pw.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                                if go {
                                    acts.push(if self.signup { Act::SignUp } else { Act::SignIn });
                                }
                                ui.label(RichText::new("Preview backend: single sign-on (SAML) is not enabled; data is held in a development database with no uptime guarantee.").size(11.0).color(p().ink4));
                            });
                            let rect = r.response.rect;
                            ui.painter().rect_filled(egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), 3.0)), 0.0, tx());
                        });
                    });
                    return;
                }
                card(ui, |ui| { egui::Frame::new().inner_margin(egui::Margin::same(12)).show(ui, |ui| {
                    ui.horizontal(|ui| { ui.label(RichText::new(&cl.name()).strong().size(15.0)); ui.label(RichText::new(&cl.email()).color(mute())); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { if ui.button("Sign out").clicked() { acts.push(Act::SignOut); } if ui.button("Refresh").clicked() { acts.push(Act::Reload); } }); });
                    ui.separator(); ui.label(RichText::new("Two-factor authentication (authenticator app)").strong());
                    if self.mfa_secret.is_empty() { if ui.button("Set up authenticator app").clicked() { acts.push(Act::MfaEnroll); } }
                    else { ui.label(RichText::new(format!("Add this secret to your authenticator app: {}", self.mfa_secret)).font(FontId::monospace(12.0))); ui.horizontal(|ui| { ui.add(egui::TextEdit::singleline(&mut self.mfa_code).hint_text("6-digit code").desired_width(110.0)); if ui.button("Verify and enable").clicked() { acts.push(Act::MfaVerify); } }); }
                }); });
                if !cl.factor().is_empty() && !cl.aal2() {
                    ui.add_space(6.0);
                    card(ui, |ui| { egui::Frame::new().inner_margin(egui::Margin::same(12)).show(ui, |ui| {
                        ui.label(RichText::new("Verify two-factor authentication").strong());
                        ui.label(RichText::new("Enter the code from your authenticator app to check or approve revisions in this session.").size(11.5).color(mute()));
                        ui.horizontal(|ui| { let r = ui.add(egui::TextEdit::singleline(&mut self.step_code).hint_text("6-digit code").desired_width(110.0)); if ui.button("Verify").clicked() || (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) { acts.push(Act::StepUp); } });
                    }); });
                }
                ui.add_space(6.0);
                card(ui, |ui| { egui::Frame::new().inner_margin(egui::Margin::same(12)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Organisation").strong());
                        let cur = self.orgs.get(self.org).map(|o| format!("{} ({})", o.1, role_label(&o.2))).unwrap_or("none".into());
                        egui::ComboBox::from_id_salt("acorg").selected_text(cur).show_ui(ui, |ui| { for i in 0..self.orgs.len() { let l = format!("{} ({})", self.orgs[i].1, role_label(&self.orgs[i].2)); if ui.selectable_value(&mut self.org, i, l).changed() { acts.push(Act::Reload); } } });
                        ui.add(egui::TextEdit::singleline(&mut self.new_org).hint_text("New organisation name").desired_width(200.0));
                        if ui.button("Create").clicked() { acts.push(Act::CreateOrg); }
                    });
                    if self.orgs.is_empty() { ui.label(RichText::new("Create an organisation to start. You become its owner.").color(mute())); }
                }); });
                if self.orgs.is_empty() { return; }
                ui.add_space(6.0);
                card(ui, |ui| { egui::Frame::new().inner_margin(egui::Margin::same(12)).show(ui, |ui| {
                    ui.label(RichText::new(format!("Members and roles - your role: {}", role_label(&self.role()))).strong());
                    let admin = self.is_admin(); let me = cl.user_id().clone();
                    egui::Grid::new("acmem").num_columns(4).spacing([14.0, 6.0]).show(ui, |ui| {
                        for m in &self.members {
                            let uid = s(m, "user_id"); let role = s(m, "role");
                            ui.label(s(m, "display_name")); ui.label(RichText::new(s(m, "email")).color(mute()));
                            let mut sel = role.clone();
                            ui.add_enabled_ui(admin, |ui| { egui::ComboBox::from_id_salt(format!("r{uid}")).selected_text(role_label(&sel)).show_ui(ui, |ui| { for (v, l) in ROLES { ui.selectable_value(&mut sel, v.to_string(), l); } }); });
                            if sel != role { acts.push(Act::SetRole(uid.clone(), sel)); }
                            if (admin || uid == me) && ui.button(if uid == me { "Leave" } else { "Remove" }).clicked() { let who = s(m, "display_name"); self.confirm = Some((if uid == me { "Leave this organisation? You lose access to its projects.".to_string() } else { format!("Remove {} from this organisation?", if who.is_empty() { s(m, "email") } else { who }) }, Act::Remove(uid.clone()))); } else { ui.label(""); }
                            ui.end_row();
                        }
                    });
                    if admin { ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut self.add_email).hint_text("Email of a registered user").desired_width(240.0));
                        egui::ComboBox::from_id_salt("acaddrole").selected_text(ROLES[self.add_role].1).show_ui(ui, |ui| { for (i, (_, l)) in ROLES.iter().enumerate() { ui.selectable_value(&mut self.add_role, i, *l); } });
                        if ui.button("Add member").clicked() { acts.push(Act::AddMember); }
                    }); }
                }); });
                ui.add_space(6.0);
                card(ui, |ui| { egui::Frame::new().inner_margin(egui::Margin::same(12)).show(ui, |ui| {
                    ui.label(RichText::new("Security and data").strong());
                    let mut req = self.orgs.get(self.org).map(|o| o.3).unwrap_or(true);
                    ui.add_enabled_ui(self.is_admin(), |ui| { if ui.checkbox(&mut req, "Require two-factor authentication to check or approve revisions").changed() { acts.push(Act::SetMfaRule(req)); } });
                    if matches!(self.role().as_str(), "owner" | "administrator" | "auditor") && ui.button("Export all organisation data to Downloads (.json)").clicked() { acts.push(Act::Export); }
                }); });
                ui.add_space(6.0);
                card(ui, |ui| { egui::Frame::new().inner_margin(egui::Margin::same(12)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Projects").strong());
                        let cur = self.projects.get(self.proj).map(|p| s(p, "name")).unwrap_or("none".into());
                        egui::ComboBox::from_id_salt("acproj").selected_text(cur).show_ui(ui, |ui| { for i in 0..self.projects.len() { let n = s(&self.projects[i], "name"); if ui.selectable_value(&mut self.proj, i, n).changed() { acts.push(Act::LoadRevs); } } });
                        if let Some(p) = self.projects.get(self.proj) { if ui.add(egui::Button::new(RichText::new("Open in Project page").color(on_ink())).fill(tx())).clicked() { open = Some((s(p, "id"), s(p, "name"))); } }
                        ui.add(egui::TextEdit::singleline(&mut self.new_proj).hint_text("New project name").desired_width(200.0));
                        if ui.button("Create").clicked() { acts.push(Act::CreateProj); }
                    });
                    if !self.revs.is_empty() || !self.projects.is_empty() {
                        ui.separator(); ui.label(RichText::new("Revisions and approval").strong());
                        ui.label(RichText::new("The designer submits, a different person with the Checker role reviews, and a third person with the Approver role signs off. Approved revisions are permanently locked.").size(11.5).color(mute()));
                    }
                    let role = self.role(); let me = cl.user_id().clone();
                    for r in self.revs.clone() {
                        let rid = s(&r, "id"); let st = s(&r, "status"); let mine = s(&r, "created_by") == me;
                        egui::Frame::new().stroke(Stroke::new(1.0_f32, line())).inner_margin(egui::Margin::same(8)).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(s(&r, "name")).strong());
                                let c = match st.as_str() { "approved" => green(), "rejected" => red(), "checked" => blue(), _ => amber() };
                                ui.label(RichText::new(status_label(&st)).strong().color(c));
                                ui.label(RichText::new(format!("hash {}", &s(&r, "content_hash")[..10.min(s(&r, "content_hash").len())])).font(FontId::monospace(11.0)).color(mute()));
                                ui.label(RichText::new(format!("by {} · {}", self.who(&s(&r, "created_by")), s(&r, "created_at"))).size(11.5).color(mute()));
                                if (st == "draft" || st == "changes_requested") && matches!(role.as_str(), "owner" | "administrator" | "project_manager" | "designer" | "student") && ui.button("Submit for checking").clicked() { acts.push(Act::Submit(rid.clone())); }
                            });
                            for x in self.reviews.iter().filter(|x| s(x, "revision_id") == rid) { ui.label(RichText::new(format!("{} · {} · {}{}", self.who(&s(x, "reviewer")), s(x, "kind").replace('_', " "), s(x, "created_at"), if s(x, "comment").is_empty() { String::new() } else { format!(" - {}", s(x, "comment")) })).size(11.5)); }
                            if !mine && ((role == "checker" && (st == "submitted" || st == "checked")) || (role == "approver" && st == "checked")) {
                                ui.horizontal(|ui| {
                                    ui.add(egui::TextEdit::singleline(&mut self.comment).hint_text("Comment (required to reject or request changes)").desired_width(300.0));
                                    if role == "checker" { for (o, l) in [("comment", "Comment"), ("recommend_approval", "Recommend approval"), ("request_changes", "Request changes"), ("reject", "Reject")] { if ui.button(l).clicked() { acts.push(Act::Check(rid.clone(), o)); } } }
                                    if role == "approver" { if ui.add(egui::Button::new(RichText::new("Approve").color(on_ink())).fill(tx())).clicked() { self.confirm = Some((format!("Approve and permanently lock {}?", s(&r, "name")), Act::Approve(rid.clone(), true))); } if ui.button("Reject").clicked() { acts.push(Act::Approve(rid.clone(), false)); } }
                                });
                            }
                        });
                        ui.add_space(4.0);
                    }
                }); });
                if !self.audit.is_empty() || matches!(self.role().as_str(), "owner" | "administrator" | "auditor") {
                    ui.add_space(6.0);
                    card(ui, |ui| { egui::Frame::new().inner_margin(egui::Margin::same(12)).show(ui, |ui| {
                        ui.horizontal(|ui| { ui.label(RichText::new("Audit trail").strong()); if ui.button("Verify integrity").clicked() { acts.push(Act::Verify); } if !self.chain.is_empty() { ui.label(RichText::new(&self.chain).strong().color(if self.chain.starts_with("Chain intact") { green() } else { red() })); } });
                        egui::Grid::new("acaud").num_columns(4).spacing([12.0, 3.0]).show(ui, |ui| { for a in &self.audit { ui.label(RichText::new(s(a, "created_at")).size(11.0).color(mute())); ui.label(self.who(&s(a, "actor"))); ui.label(RichText::new(s(a, "action")).strong()); ui.label(RichText::new(a["detail"].to_string()).font(FontId::monospace(10.5)).color(mute())); ui.end_row(); } });
                    }); });
                }
                ui.add_space(8.0);
                card(ui, |ui| { egui::Frame::new().inner_margin(egui::Margin::same(12)).show(ui, |ui| {
                    ui.label(RichText::new("Close my account").strong());
                    ui.label(RichText::new("Removes you from all organisations and disables sign-in. Engineering records you created are kept for audit. You must not be the only owner of an organisation.").size(11.5).color(mute()));
                    ui.checkbox(&mut self.close_ok, "I understand this cannot be undone");
                    if ui.add_enabled(self.close_ok, egui::Button::new("Close account")).clicked() { acts.push(Act::CloseAccount); }
                }); });
                ui.add_space(8.0);
                ui.label(RichText::new("Preview backend. Single sign-on (SAML) is not enabled. Data is held in a development database on a free plan.").size(11.5).color(amber()));
            });
        });
        if !acts.is_empty() && !busy {
            // Input checks that need no network run here so their message appears immediately.
            if let Some(e) = acts.iter().find_map(|a| self.precheck(a)) {
                self.msg.clear();
                self.err = e;
                return open;
            }
            let (tx, rx) = channel();
            job.rx = Some(rx);
            let (mut a, mut c) = (self.clone(), cl.clone());
            let ctx2 = ctx.clone();
            std::thread::spawn(move || {
                a.apply(acts, &mut c);
                let _ = tx.send((a, c));
                ctx2.request_repaint();
            });
        }
        open
    }

    fn precheck(&self, a: &Act) -> Option<String> {
        let email_ok = |e: &str| {
            let e = e.trim();
            e.len() > 4 && e.contains('@') && e.rsplit('@').next().is_some_and(|d| d.contains('.')) && !e.contains(' ')
        };
        let code_ok = |c: &str| {
            let c = c.trim();
            c.len() == 6 && c.chars().all(|x| x.is_ascii_digit())
        };
        match a {
            Act::SignIn | Act::SignUp if !email_ok(&self.email) => Some("Enter a valid email address.".into()),
            Act::SignIn | Act::SignUp if self.pw.chars().count() < 8 => Some("The password must be at least 8 characters.".into()),
            Act::SignUp if self.name.trim().is_empty() => {
                Some("Enter a display name. It is shown to your team on reviews and approvals.".into())
            }
            Act::CreateOrg if self.new_org.trim().is_empty() => Some("Enter an organisation name.".into()),
            Act::CreateProj if self.new_proj.trim().is_empty() => Some("Enter a project name.".into()),
            Act::AddMember if !email_ok(&self.add_email) => Some("Enter the email address of a registered user.".into()),
            Act::StepUp if !code_ok(&self.step_code) => Some("Enter the 6-digit code from your authenticator app.".into()),
            Act::MfaVerify if !code_ok(&self.mfa_code) => Some("Enter the 6-digit code from your authenticator app.".into()),
            Act::Check(_, o) if matches!(*o, "reject" | "request_changes") && self.comment.trim().is_empty() => {
                Some("A comment is required to reject or request changes.".into())
            }
            Act::Approve(_, false) if self.comment.trim().is_empty() => Some("A comment is required to reject.".into()),
            _ => None,
        }
    }

    /// Performs actions against the backend. Runs on a worker thread.
    fn apply(&mut self, acts: Vec<Act>, cl: &mut Client) {
        let org = self.orgs.get(self.org).map(|o| o.0.clone()).unwrap_or_default();
        for a in acts {
            match a {
                Act::LoadAll => {
                    self.load_orgs(cl);
                }
                Act::LoadRevs => {
                    self.load_revs(cl);
                }
                Act::StepUp => {
                    let r = cl.mfa_step_up(self.step_code.trim());
                    if self.ok(r, "Two-factor verified for this session.").is_some() {
                        self.step_code.clear();
                    }
                }
                Act::SetMfaRule(v) => {
                    let r = cl.rpc("set_org_security", &json!({"p_org": org, "p_require_mfa": v}));
                    self.ok(r, "Security setting updated.");
                    self.load_orgs(cl);
                }
                Act::Export => {
                    let r = cl.rpc("export_organisation", &json!({"p_org": org}));
                    if let Some(v) = self.ok(r, "") {
                        let dir = user_dir(&["Downloads"]);
                        let name = self.orgs.get(self.org).map(|o| o.1.clone()).unwrap_or("organisation".into());
                        let p = dir.join(format!("strucal-{}-export.json", name.replace(|c: char| !c.is_alphanumeric(), "-")));
                        match std::fs::write(&p, serde_json::to_string_pretty(&v).unwrap_or_default()) {
                            Ok(_) => {
                                self.msg = format!("Exported to Downloads: {}", p.file_name().and_then(|x| x.to_str()).unwrap_or(""));
                            }
                            Err(e) => {
                                self.err = format!("Export failed: {e}");
                            }
                        }
                    }
                }
                Act::CloseAccount => {
                    let r = cl.rpc("close_account", &json!({}));
                    if self.ok(r, "Account closed.").is_some() {
                        cl.sign_out();
                        *self = Acct::new();
                    }
                }
                Act::SignIn => {
                    let r = cl.sign_in(self.email.trim(), &self.pw);
                    if self.ok(r, "Signed in.").is_some() {
                        self.pw.clear();
                    }
                }
                Act::SignUp => {
                    let r = cl.sign_up(self.email.trim(), &self.pw, &self.name);
                    if self.ok(r, "Account created and signed in.").is_some() {
                        self.pw.clear();
                    }
                }
                Act::SignOut => {
                    cl.sign_out();
                    *self = Acct::new();
                }
                Act::Reload => {
                    self.load_org(cl);
                }
                Act::CreateOrg => {
                    let r = cl.rpc("create_organisation", &json!({"p_name": self.new_org.trim()}));
                    if self.ok(r, "Organisation created.").is_some() {
                        self.new_org.clear();
                        self.load_orgs(cl);
                    }
                }
                Act::CreateProj => {
                    let r = cl.rpc("create_project", &json!({"p_org": org, "p_name": self.new_proj.trim(), "p_description": ""}));
                    if self.ok(r, "Project created.").is_some() {
                        self.new_proj.clear();
                        self.load_org(cl);
                    }
                }
                Act::AddMember => {
                    let r =
                        cl.rpc("add_member", &json!({"p_org": org, "p_email": self.add_email.trim(), "p_role": ROLES[self.add_role].0}));
                    if self.ok(r, "Member added.").is_some() {
                        self.add_email.clear();
                        self.load_org(cl);
                    }
                }
                Act::SetRole(u, role) => {
                    let r = cl.rpc("set_member_role", &json!({"p_org": org, "p_user": u, "p_role": role}));
                    self.ok(r, "Role updated.");
                    self.load_orgs(cl);
                }
                Act::Remove(u) => {
                    let me = u == cl.user_id();
                    let r = cl.rpc("remove_member", &json!({"p_org": org, "p_user": u}));
                    if self.ok(r, "Member removed.").is_some() {
                        if me {
                            self.load_orgs(cl);
                        } else {
                            self.load_org(cl);
                        }
                    }
                }
                Act::Submit(rev) => {
                    let r = cl.rpc("submit_revision", &json!({"p_rev": rev}));
                    self.ok(r, "Submitted for checking.");
                    self.load_revs(cl);
                }
                Act::Check(rev, o) => {
                    let r = cl.rpc("check_revision", &json!({"p_rev": rev, "p_outcome": o, "p_comment": self.comment}));
                    if self.ok(r, "Review recorded.").is_some() {
                        self.comment.clear();
                    }
                    self.load_org(cl);
                }
                Act::Approve(rev, yes) => {
                    let r = cl.rpc("approve_revision", &json!({"p_rev": rev, "p_approve": yes, "p_comment": self.comment}));
                    if self.ok(r, if yes { "Revision approved and locked." } else { "Revision rejected." }).is_some() {
                        self.comment.clear();
                    }
                    self.load_org(cl);
                }
                Act::Verify => {
                    let r = cl.rpc("verify_audit_chain", &json!({"p_org": org}));
                    if let Some(v) = self.ok(r, "") {
                        let row = v.as_array().and_then(|a| a.first()).cloned().unwrap_or(Value::Null);
                        self.chain = if row["ok"].as_bool() == Some(true) {
                            format!("Chain intact ({} events)", row["checked"])
                        } else {
                            format!("TAMPERING DETECTED at event {}", row["broken_at"])
                        };
                    }
                }
                Act::MfaEnroll => {
                    let r = cl.mfa_enroll();
                    if let Some(v) = self.ok(r, "Add the secret to your authenticator app, then enter a code.") {
                        self.mfa_factor = s(&v, "id");
                        self.mfa_secret = s(&v["totp"], "secret");
                    }
                }
                Act::MfaVerify => {
                    let r = cl.mfa_verify(&self.mfa_factor, self.mfa_code.trim());
                    if self.ok(r, "Two-factor authentication enabled.").is_some() {
                        self.mfa_secret.clear();
                        self.mfa_code.clear();
                    }
                }
            }
        }
    }
}
