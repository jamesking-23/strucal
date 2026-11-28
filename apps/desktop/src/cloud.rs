//! Minimal Supabase client (Auth + PostgREST) for the desktop app. The key is the public publishable key; access
//! is enforced by row-level security.
//!
//! Session model:
//! - One session is shared by every clone of `Client` (`Arc<Mutex<…>>`), so a token refresh performed by a
//!   background job is immediately visible to the UI and to other jobs. Supabase rotates refresh tokens on every
//!   refresh; sharing avoids clones holding a spent token.
//! - Every change to the tokens is persisted with `secure_store` (Windows Credential Manager, or a 0600 file), so
//!   the user stays signed in across restarts. `restore()` re-establishes the session at launch.
//! - A network failure never signs the user out. Only a refresh rejected by the server (or an explicit sign-out)
//!   clears the stored session.
use crate::secure_store;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub const URL: &str = "https://bunshysrzoozplnvxhui.supabase.co";
pub const KEY: &str = "sb_publishable_Z1waCuLYUVRaGV9LyWxLAw_gbl-HLb9";

#[derive(Default, Clone)]
struct Session {
    token: String,
    refresh: String,
    user_id: String,
    email: String,
    name: String,
    factor: String,
    aal2: bool,
    /// Access-token expiry (unix seconds); 0 when unknown.
    expires_at: u64,
}

/// Outcome of restoring a stored session at launch.
pub enum Restore {
    /// No stored session.
    None,
    /// Signed in with a fresh access token.
    Restored,
    /// A stored session exists but the server could not be reached; it is kept and retried later.
    Offline(String),
    /// The stored session was rejected (expired or revoked) and has been removed.
    Expired,
}

#[derive(Default, Clone)]
pub struct Client {
    s: Arc<Mutex<Session>>,
}

/// Error of a single HTTP call: status 0 means the server was not reached.
type CallErr = (u16, String);

fn clean(code: u16, body: &str) -> String {
    let v: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    for k in ["message", "msg", "error_description", "error"] {
        if let Some(s) = v[k].as_str() {
            return s.to_string();
        }
    }
    format!("Request failed ({code})")
}

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// Decodes the payload of a JWT (base64url, no signature check — used only for expiry and assurance level).
fn jwt_claims(token: &str) -> Value {
    let Some(payload) = token.split('.').nth(1) else { return Value::Null };
    let mut bits: u32 = 0;
    let mut n = 0;
    let mut out = Vec::with_capacity(payload.len() * 3 / 4);
    for c in payload.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'-' | b'+' => 62,
            b'_' | b'/' => 63,
            _ => continue,
        } as u32;
        bits = (bits << 6) | v;
        n += 6;
        if n >= 8 {
            n -= 8;
            out.push((bits >> n) as u8);
            bits &= (1 << n) - 1;
        }
    }
    serde_json::from_slice(&out).unwrap_or(Value::Null)
}

impl Client {
    fn lock(&self) -> std::sync::MutexGuard<'_, Session> {
        self.s.lock().unwrap_or_else(|p| p.into_inner())
    }

    pub fn signed_in(&self) -> bool {
        !self.lock().token.is_empty()
    }
    /// True when a stored session exists but has not been confirmed with the server yet (offline launch).
    pub fn has_pending_session(&self) -> bool {
        let s = self.lock();
        s.token.is_empty() && !s.refresh.is_empty()
    }
    pub fn user_id(&self) -> String {
        self.lock().user_id.clone()
    }
    pub fn email(&self) -> String {
        self.lock().email.clone()
    }
    pub fn name(&self) -> String {
        self.lock().name.clone()
    }
    pub fn factor(&self) -> String {
        self.lock().factor.clone()
    }
    pub fn aal2(&self) -> bool {
        self.lock().aal2
    }

    fn call(&self, method: &str, path: &str, body: Option<&Value>, extra: &[(&str, &str)]) -> Result<Value, CallErr> {
        let token = self.lock().token.clone();
        let mut r = ureq::request(method, &format!("{URL}{path}"))
            .set("apikey", KEY)
            .set("Content-Type", "application/json")
            .timeout(Duration::from_secs(20));
        if !token.is_empty() {
            r = r.set("Authorization", &format!("Bearer {token}"));
        }
        for (k, v) in extra {
            r = r.set(k, v);
        }
        let res = match body {
            Some(b) => r.send_json(b.clone()),
            None => r.call(),
        };
        match res {
            Ok(resp) => {
                let t = resp.into_string().unwrap_or_default();
                Ok(serde_json::from_str(&t).unwrap_or(Value::Null))
            }
            Err(ureq::Error::Status(code, resp)) => Err((code, clean(code, &resp.into_string().unwrap_or_default()))),
            Err(e) => Err((0, format!("Cannot reach the server ({e}). Check your connection."))),
        }
    }

    /// Applies a token response and persists it.
    fn set_session(&self, v: &Value) {
        {
            let mut s = self.lock();
            s.token = v["access_token"].as_str().unwrap_or("").into();
            if let Some(r) = v["refresh_token"].as_str() {
                s.refresh = r.into();
            }
            if let Some(id) = v["user"]["id"].as_str() {
                s.user_id = id.into();
            }
            if let Some(e) = v["user"]["email"].as_str() {
                s.email = e.into();
            }
            let fallback = s.email.clone();
            if v["user"].is_object() {
                s.name = v["user"]["user_metadata"]["display_name"].as_str().unwrap_or(&fallback).into();
            }
            if let Some(fs) = v["user"]["factors"].as_array() {
                s.factor = fs
                    .iter()
                    .find(|f| f["factor_type"] == "totp" && f["status"] == "verified")
                    .and_then(|f| f["id"].as_str())
                    .unwrap_or("")
                    .into();
            }
            let claims = jwt_claims(&s.token);
            s.aal2 = claims["aal"].as_str() == Some("aal2");
            s.expires_at = claims["exp"].as_u64().unwrap_or(0);
        }
        self.persist();
    }

    /// Saves the refresh token and profile so the session survives a restart.
    fn persist(&self) {
        let s = self.lock().clone();
        if s.refresh.is_empty() {
            return;
        }
        let blob = json!({"refresh": s.refresh, "user_id": s.user_id, "email": s.email, "name": s.name, "factor": s.factor}).to_string();
        let _ = secure_store::save_session(&blob);
    }

    /// Loads a stored session (without contacting the server). Call `restore()` afterwards to validate it.
    pub fn load_stored(&self) -> bool {
        let Some(v) = secure_store::load_session().and_then(|s| serde_json::from_str::<Value>(&s).ok()) else { return false };
        let mut s = self.lock();
        s.refresh = v["refresh"].as_str().unwrap_or("").into();
        s.user_id = v["user_id"].as_str().unwrap_or("").into();
        s.email = v["email"].as_str().unwrap_or("").into();
        s.name = v["name"].as_str().unwrap_or("").into();
        s.factor = v["factor"].as_str().unwrap_or("").into();
        !s.refresh.is_empty()
    }

    /// Exchanges the stored refresh token for a fresh access token. Safe to call from a background thread.
    pub fn restore(&self) -> Restore {
        if self.lock().refresh.is_empty() && !self.load_stored() {
            return Restore::None;
        }
        match self.refresh_session() {
            Ok(()) => Restore::Restored,
            Err((0, e)) => Restore::Offline(e),
            Err(_) => {
                self.sign_out();
                Restore::Expired
            }
        }
    }

    pub fn sign_in(&self, email: &str, pw: &str) -> Result<(), String> {
        let v = self
            .call("POST", "/auth/v1/token?grant_type=password", Some(&json!({"email": email, "password": pw})), &[])
            .map_err(|e| e.1)?;
        self.set_session(&v);
        Ok(())
    }
    pub fn sign_up(&self, email: &str, pw: &str, name: &str) -> Result<(), String> {
        self.call("POST", "/auth/v1/signup", Some(&json!({"email": email, "password": pw, "data": {"display_name": name}})), &[])
            .map_err(|e| e.1)?;
        self.sign_in(email, pw).map_err(|e| {
            format!("Account created, but sign-in failed ({e}). If email confirmation is on, click the link in your email first.")
        })
    }
    /// Signs out everywhere in the app (all clones share the session) and removes the stored session.
    pub fn sign_out(&self) {
        let token = {
            let mut s = self.lock();
            let t = std::mem::take(&mut s.token);
            *s = Session::default();
            t
        };
        secure_store::clear_session();
        if !token.is_empty() {
            // Best effort: revoke the refresh token server-side without blocking the caller.
            std::thread::spawn(move || {
                let _ = ureq::post(&format!("{URL}/auth/v1/logout"))
                    .set("apikey", KEY)
                    .set("Authorization", &format!("Bearer {token}"))
                    .timeout(Duration::from_secs(5))
                    .call();
            });
        }
    }

    fn refresh_session(&self) -> Result<(), CallErr> {
        let refresh = self.lock().refresh.clone();
        if refresh.is_empty() {
            return Err((401, "No session".into()));
        }
        let v = self.call("POST", "/auth/v1/token?grant_type=refresh_token", Some(&json!({"refresh_token": refresh})), &[])?;
        if !v["access_token"].is_string() {
            return Err((401, "The session could not be renewed".into()));
        }
        // Two jobs refreshing at the same moment both succeed: Supabase accepts a just-rotated token within its reuse interval.
        self.set_session(&v);
        Ok(())
    }

    /// Authenticated request: refreshes ahead of expiry and once more on 401. Never signs out on a network error.
    fn req(&self, method: &str, path: &str, body: Option<&Value>, extra: &[(&str, &str)]) -> Result<Value, String> {
        let (needs, expires_at) = {
            let s = self.lock();
            (s.token.is_empty() && !s.refresh.is_empty(), s.expires_at)
        };
        if needs || (expires_at > 0 && expires_at <= now() + 60) {
            match self.refresh_session() {
                Ok(()) => {}
                Err((0, e)) => return Err(e),
                Err(_) => {
                    self.sign_out();
                    return Err("Your session has expired. Please sign in again.".into());
                }
            }
        }
        match self.call(method, path, body, extra) {
            Err((401, msg)) => match self.refresh_session() {
                Ok(()) => self.call(method, path, body, extra).map_err(|e| e.1),
                Err((0, e)) => Err(e),
                Err(_) => {
                    self.sign_out();
                    Err(format!("Your session has expired. Please sign in again. ({msg})"))
                }
            },
            other => other.map_err(|e| e.1),
        }
    }

    pub fn get(&self, path: &str) -> Result<Value, String> {
        self.req("GET", &format!("/rest/v1/{path}"), None, &[])
    }
    pub fn rpc(&self, name: &str, args: &Value) -> Result<Value, String> {
        self.req("POST", &format!("/rest/v1/rpc/{name}"), Some(args), &[])
    }
    pub fn patch(&self, path: &str, body: &Value) -> Result<Value, String> {
        self.req("PATCH", &format!("/rest/v1/{path}"), Some(body), &[("Prefer", "return=representation")])
    }
    pub fn function(&self, name: &str, body: &Value) -> Result<Value, String> {
        self.req("POST", &format!("/functions/v1/{name}"), Some(body), &[])
    }
    pub fn mfa_enroll(&self) -> Result<Value, String> {
        self.req("POST", "/auth/v1/factors", Some(&json!({"factor_type": "totp"})), &[])
    }
    /// Verifies a code for a factor. The response is a fresh session at assurance level 2.
    pub fn mfa_verify(&self, factor: &str, code: &str) -> Result<(), String> {
        let ch = self.req("POST", &format!("/auth/v1/factors/{factor}/challenge"), Some(&json!({})), &[])?;
        let cid = ch["id"].as_str().ok_or("No challenge returned")?.to_string();
        let v = self.req("POST", &format!("/auth/v1/factors/{factor}/verify"), Some(&json!({"challenge_id": cid, "code": code})), &[])?;
        if v["access_token"].is_string() {
            self.set_session(&v);
            let mut s = self.lock();
            s.factor = factor.into();
            s.aal2 = true;
            drop(s);
            self.persist();
        }
        Ok(())
    }
    pub fn mfa_step_up(&self, code: &str) -> Result<(), String> {
        let f = self.factor();
        if f.is_empty() {
            return Err("No authenticator app is set up".into());
        }
        self.mfa_verify(&f, code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decodes_jwt_claims() {
        // header.payload.signature with payload {"exp":1700000000,"aal":"aal2"}
        let token = "eyJhbGciOiJIUzI1NiJ9.eyJleHAiOjE3MDAwMDAwMDAsImFhbCI6ImFhbDIifQ.sig";
        let c = jwt_claims(token);
        assert_eq!(c["exp"].as_u64(), Some(1_700_000_000));
        assert_eq!(c["aal"].as_str(), Some("aal2"));
    }
    #[test]
    fn malformed_jwt_is_null() {
        assert!(jwt_claims("not-a-token").is_null());
    }
}
