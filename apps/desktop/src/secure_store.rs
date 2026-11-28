//! Persistent storage for the signed-in session and for local app preferences.
//!
//! The session (refresh token and profile) is a credential: on Windows it is kept in the Windows Credential
//! Manager through the `keyring` crate; on other platforms it is written to a file readable only by the
//! current user (mode 0600). Preferences (workspace inputs, theme) are plain JSON in the app's config folder.
use std::path::PathBuf;

const SERVICE: &str = "StruCal";
const ACCOUNT: &str = "supabase-session";

/// Per-user configuration folder: `%APPDATA%\StruCal` on Windows, `$XDG_CONFIG_HOME/strucal` or `~/.config/strucal` elsewhere.
pub fn config_dir() -> PathBuf {
    let base = if cfg!(windows) {
        std::env::var("APPDATA").ok().filter(|v| !v.is_empty()).map(PathBuf::from).unwrap_or_else(|| home().join("AppData").join("Roaming"))
    } else {
        std::env::var("XDG_CONFIG_HOME").ok().filter(|v| !v.is_empty()).map(PathBuf::from).unwrap_or_else(|| home().join(".config"))
    };
    let dir = base.join(if cfg!(windows) { "StruCal" } else { "strucal" });
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn home() -> PathBuf {
    std::env::var("USERPROFILE")
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| std::env::var("HOME").ok())
        .map(PathBuf::from)
        .unwrap_or_default()
}

// ---------------------------------------------------------------- session secret

#[cfg(windows)]
mod imp {
    use super::{ACCOUNT, SERVICE};
    fn entry() -> Option<keyring::Entry> {
        keyring::Entry::new(SERVICE, ACCOUNT).ok()
    }
    pub fn load() -> Option<String> {
        entry()?.get_password().ok()
    }
    pub fn save(secret: &str) -> Result<(), String> {
        entry().ok_or("Credential Manager is unavailable")?.set_password(secret).map_err(|e| e.to_string())
    }
    pub fn clear() {
        if let Some(e) = entry() {
            let _ = e.delete_credential();
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use super::config_dir;
    use std::io::Write;
    fn path() -> std::path::PathBuf {
        config_dir().join(format!("{}.json", super::ACCOUNT))
    }
    pub fn load() -> Option<String> {
        let _ = super::SERVICE;
        std::fs::read_to_string(path()).ok().filter(|s| !s.trim().is_empty())
    }
    pub fn save(secret: &str) -> Result<(), String> {
        let p = path();
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let mut f = opts.open(&p).map_err(|e| e.to_string())?;
        f.write_all(secret.as_bytes()).map_err(|e| e.to_string())
    }
    pub fn clear() {
        let _ = std::fs::remove_file(path());
    }
}

/// Returns the stored session secret, if any.
pub fn load_session() -> Option<String> {
    imp::load()
}
/// Stores the session secret, replacing any previous one.
pub fn save_session(secret: &str) -> Result<(), String> {
    imp::save(secret)
}
/// Removes the stored session (sign-out or rejected token).
pub fn clear_session() {
    imp::clear()
}

// ---------------------------------------------------------------- preferences

/// Reads a JSON preferences file from the config folder.
pub fn load_prefs(name: &str) -> Option<serde_json::Value> {
    let s = std::fs::read_to_string(config_dir().join(name)).ok()?;
    serde_json::from_str(&s).ok()
}
/// Writes a JSON preferences file atomically (write to a temporary file, then rename).
pub fn save_prefs(name: &str, v: &serde_json::Value) {
    let dir = config_dir();
    let tmp = dir.join(format!("{name}.tmp"));
    if std::fs::write(&tmp, serde_json::to_string_pretty(v).unwrap_or_default()).is_ok() {
        let _ = std::fs::rename(&tmp, dir.join(name));
    }
}
