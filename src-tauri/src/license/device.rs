/// Generates a stable machine identifier that survives reboots and OS updates
/// but changes on full hardware replacement.
///
/// Strategy: hash multiple stable sources (hostname + platform ID).
/// We accept partial matches gracefully — a church replacing one component
/// shouldn't lose their activation; they use the Zehut portal to re-activate.

use sha2::{Digest, Sha256};
use std::path::Path;

pub fn machine_id(app_data_dir: &Path) -> String {
    let mut hasher = Sha256::new();

    // Hostname — changes rarely in a production media workstation
    if let Ok(h) = hostname() {
        hasher.update(h.as_bytes());
    }

    // Platform-stable hardware/install ID
    if let Some(hw) = platform_id() {
        hasher.update(hw.as_bytes());
    }

    // Fallback: a UUID persisted in app data (survives as long as the app)
    let install_id = load_or_create_install_id(app_data_dir);
    hasher.update(install_id.as_bytes());

    let hash = hasher.finalize();
    // 16 hex chars — short enough to display, unique enough to identify
    format!("{:x}", hash)[..16].to_string()
}

// ── platform implementations ──────────────────────────────────────────────────

fn hostname() -> Result<String, ()> {
    std::process::Command::new("hostname")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .map_err(|_| ())
}

#[cfg(target_os = "linux")]
fn platform_id() -> Option<String> {
    std::fs::read_to_string("/etc/machine-id")
        .ok()
        .map(|s| s.trim().to_string())
}

#[cfg(target_os = "macos")]
fn platform_id() -> Option<String> {
    std::process::Command::new("ioreg")
        .args(["-rd1", "-c", "IOPlatformExpertDevice"])
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout);
            // Extract IOPlatformUUID value
            s.lines()
                .find(|l| l.contains("IOPlatformUUID"))
                .and_then(|l| {
                    let start = l.find('"')? + 1;
                    let end = l.rfind('"')?;
                    if start < end { Some(l[start..end].to_string()) } else { None }
                })
        })
}

#[cfg(target_os = "windows")]
fn platform_id() -> Option<String> {
    std::process::Command::new("powershell")
        .args(["-Command", "(Get-ItemProperty -Path 'HKLM:\\SOFTWARE\\Microsoft\\Cryptography' -Name MachineGuid).MachineGuid"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn platform_id() -> Option<String> { None }

// ── install-scoped UUID fallback ──────────────────────────────────────────────

fn load_or_create_install_id(dir: &Path) -> String {
    let path = dir.join(".install_id");
    if let Ok(id) = std::fs::read_to_string(&path) {
        let trimmed = id.trim().to_string();
        if !trimmed.is_empty() { return trimmed; }
    }
    let id = uuid::Uuid::new_v4().to_string();
    let _ = std::fs::write(&path, &id);
    id
}
