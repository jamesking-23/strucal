//! Minimal Supabase client (Auth + PostgREST) for the desktop app. The key is the public publishable key; access is enforced by row-level security.
use serde_json::{json, Value};
use std::time::Duration;

pub const URL: &str = "https://bunshysrzoozplnvxhui.supabase.co";
pub const KEY: &str = "sb_publishable_Z1waCuLYUVRaGV9LyWxLAw_gbl-HLb9";

#[derive(Default, Clone)]
pub struct Client { pub token: String, pub refresh: String, pub user_id: String, pub email: String, pub name: String }

fn clean(code: u16, body: &str) -> String {
    let v: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    for k in ["message", "msg", "error_description", "error"] { if let Some(s) = v[k].as_str() { return s.to_string(); } }
    format!("Request failed ({code})")
}

impl Client {
    pub fn signed_in(&self) -> bool { !self.token.is_empty() }
    fn call(&self, method: &str, path: &str, body: Option<&Value>, extra: &[(&str, &str)]) -> Result<Value, (u16, String)> {
        let mut r = ureq::request(method, &format!("{URL}{path}")).set("apikey", KEY).set("Content-Type", "application/json").timeout(Duration::from_secs(20));
        if !self.token.is_empty() { r = r.set("Authorization", &format!("Bearer {}", self.token)); }
        for (k, v) in extra { r = r.set(k, v); }
        let res = match body { Some(b) => r.send_json(b.clone()), None => r.call() };
        match res {
            Ok(resp) => { let t = resp.into_string().unwrap_or_default(); Ok(serde_json::from_str(&t).unwrap_or(Value::Null)) }
            Err(ureq::Error::Status(code, resp)) => Err((code, clean(code, &resp.into_string().unwrap_or_default()))),
            Err(e) => Err((0, format!("Cannot reach the server: {e}"))),
        }
    }
    fn set_session(&mut self, v: &Value) {
        self.token = v["access_token"].as_str().unwrap_or("").into(); self.refresh = v["refresh_token"].as_str().unwrap_or("").into();
        self.user_id = v["user"]["id"].as_str().unwrap_or("").into(); self.email = v["user"]["email"].as_str().unwrap_or("").into();
        self.name = v["user"]["user_metadata"]["display_name"].as_str().unwrap_or(&self.email).into();
    }
    pub fn sign_in(&mut self, email: &str, pw: &str) -> Result<(), String> {
        let v = self.call("POST", "/auth/v1/token?grant_type=password", Some(&json!({"email": email, "password": pw})), &[]).map_err(|e| e.1)?;
        self.set_session(&v); Ok(())
    }
    pub fn sign_up(&mut self, email: &str, pw: &str, name: &str) -> Result<(), String> {
        self.call("POST", "/auth/v1/signup", Some(&json!({"email": email, "password": pw, "data": {"display_name": name}})), &[]).map_err(|e| e.1)?;
        self.sign_in(email, pw).map_err(|e| format!("Account created, but sign-in failed ({e}). If email confirmation is on, click the link in your email first."))
    }
    pub fn sign_out(&mut self) { *self = Client::default(); }
    fn refresh_session(&mut self) -> bool {
        if self.refresh.is_empty() { return false; }
        match self.call("POST", "/auth/v1/token?grant_type=refresh_token", Some(&json!({"refresh_token": self.refresh})), &[]) { Ok(v) if v["access_token"].is_string() => { self.set_session(&v); true } _ => false }
    }
    fn req(&mut self, method: &str, path: &str, body: Option<&Value>, extra: &[(&str, &str)]) -> Result<Value, String> {
        match self.call(method, path, body, extra) {
            Err((401, _)) if self.refresh_session() => self.call(method, path, body, extra).map_err(|e| e.1),
            Err((401, _)) => { self.sign_out(); Err("Your session expired. Please sign in again.".into()) }
            other => other.map_err(|e| e.1),
        }
    }
    pub fn get(&mut self, path: &str) -> Result<Value, String> { self.req("GET", &format!("/rest/v1/{path}"), None, &[]) }
    pub fn rpc(&mut self, name: &str, args: &Value) -> Result<Value, String> { self.req("POST", &format!("/rest/v1/rpc/{name}"), Some(args), &[]) }
    pub fn patch(&mut self, path: &str, body: &Value) -> Result<Value, String> { self.req("PATCH", &format!("/rest/v1/{path}"), Some(body), &[("Prefer", "return=representation")]) }
    pub fn mfa_enroll(&mut self) -> Result<Value, String> { self.req("POST", "/auth/v1/factors", Some(&json!({"factor_type": "totp"})), &[]) }
    pub fn mfa_verify(&mut self, factor: &str, code: &str) -> Result<(), String> {
        let ch = self.req("POST", &format!("/auth/v1/factors/{factor}/challenge"), Some(&json!({})), &[])?;
        let cid = ch["id"].as_str().ok_or("No challenge returned")?.to_string();
        self.req("POST", &format!("/auth/v1/factors/{factor}/verify"), Some(&json!({"challenge_id": cid, "code": code})), &[]).map(|_| ())
    }
}
