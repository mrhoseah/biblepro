/// OS-keychain backed token storage via the `keyring` crate.
/// Wraps Keychain (macOS), Credential Manager (Windows), libsecret (Linux).
use keyring::Entry;

const SERVICE: &str = "biblepro";
const KEY_TOKEN: &str = "license_token";
const KEY_GRACE_START: &str = "grace_start";

fn entry(key: &str) -> Result<Entry, String> {
    Entry::new(SERVICE, key).map_err(|e| format!("keyring entry error: {e}"))
}

// ── token ─────────────────────────────────────────────────────────────────────

pub fn save_token(token: &str) -> Result<(), String> {
    entry(KEY_TOKEN)?
        .set_password(token)
        .map_err(|e| format!("keychain write failed: {e}"))
}

pub fn load_token() -> Option<String> {
    entry(KEY_TOKEN).ok()?.get_password().ok()
}

pub fn clear_token() {
    if let Ok(e) = entry(KEY_TOKEN) {
        let _ = e.delete_credential();
    }
    clear_grace_start();
}

// ── grace period tracking ─────────────────────────────────────────────────────
// When a valid (but now-expired) token is found and we can't reach the cloud,
// we record when the grace period started so we can enforce the window.

pub fn save_grace_start(unix_secs: i64) -> Result<(), String> {
    entry(KEY_GRACE_START)?
        .set_password(&unix_secs.to_string())
        .map_err(|e| format!("keychain write failed: {e}"))
}

pub fn load_grace_start() -> Option<i64> {
    entry(KEY_GRACE_START)
        .ok()?
        .get_password()
        .ok()?
        .parse()
        .ok()
}

pub fn clear_grace_start() {
    if let Ok(e) = entry(KEY_GRACE_START) {
        let _ = e.delete_credential();
    }
}
