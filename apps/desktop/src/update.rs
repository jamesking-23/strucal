//! In-app updater: checks GitHub Releases and swaps the running executable in place.
use std::io::Read;

pub const CURRENT: &str = match option_env!("STRUCAL_VERSION") { Some(v) => v, None => env!("CARGO_PKG_VERSION") };
const API: &str = "https://api.github.com/repos/jamesking-23/strucal/releases/latest";
const ASSET: &str = "strucal-desktop-windows-x64.exe";

fn parse(v: &str) -> Vec<u64> { v.trim_start_matches('v').split('.').map(|p| p.parse().unwrap_or(0)).collect() }

pub struct Available { pub version: String, pub url: String, pub notes: String }

pub fn check() -> Result<Option<Available>, String> {
    let r: serde_json::Value = ureq::get(API).set("User-Agent", "StruCal-Desktop").set("Accept", "application/vnd.github+json")
        .call().map_err(|e| format!("Could not reach the update server: {e}"))?.into_json().map_err(|e| e.to_string())?;
    let tag = r["tag_name"].as_str().ok_or("No release found")?;
    if parse(tag) <= parse(CURRENT) { return Ok(None); }
    let url = r["assets"].as_array().and_then(|a| a.iter().find(|x| x["name"] == ASSET)).and_then(|x| x["browser_download_url"].as_str())
        .ok_or("Release has no Windows installer asset yet")?;
    Ok(Some(Available { version: tag.trim_start_matches('v').to_string(), url: url.to_string(), notes: r["body"].as_str().unwrap_or("").to_string() }))
}

/// Downloads the new executable, swaps it in (a running .exe can be renamed on Windows) and relaunches.
pub fn install(url: &str) -> Result<(), String> {
    let cur = std::env::current_exe().map_err(|e| e.to_string())?;
    let new = cur.with_extension("new");
    let old = cur.with_extension("old");
    let mut buf = Vec::new();
    ureq::get(url).set("User-Agent", "StruCal-Desktop").call().map_err(|e| e.to_string())?.into_reader().read_to_end(&mut buf).map_err(|e| e.to_string())?;
    if buf.len() < 100_000 || &buf[0..2] != b"MZ" { return Err("Downloaded file is not a valid Windows executable".into()); }
    std::fs::write(&new, &buf).map_err(|e| format!("Cannot write update (is the app in a protected folder?): {e}"))?;
    let _ = std::fs::remove_file(&old);
    std::fs::rename(&cur, &old).map_err(|e| e.to_string())?;
    if let Err(e) = std::fs::rename(&new, &cur) { let _ = std::fs::rename(&old, &cur); return Err(e.to_string()); }
    std::process::Command::new(&cur).spawn().map_err(|e| e.to_string())?;
    std::process::exit(0);
}

pub fn cleanup_old() { if let Ok(c) = std::env::current_exe() { let _ = std::fs::remove_file(c.with_extension("old")); } }
