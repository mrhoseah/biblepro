use serde::{Deserialize, Serialize};
use tauri::State;

use super::{
    device, keystore, now_unix, token, LicenseData, LicenseState, LicenseStatus, CLOUD_API,
};

// ── activate_license ──────────────────────────────────────────────────────────
// The "license key" the user pastes in IS the signed JWT — no extra lookup needed.
// The Zehut backend issues a JWT bound to the device_id during purchase/allocation.
// For offline activation the user receives the JWT directly.

#[tauri::command]
pub async fn activate_license(
    state: State<'_, LicenseState>,
    token_str: String,
) -> Result<LicenseStatus, String> {
    let machine = device::machine_id(&state.app_data_dir);

    let claims = token::verify_token(&token_str)?;

    if claims.device_id != machine {
        return Err(format!(
            "This license is bound to a different device ({}).\n\
             Contact support at zehut.io/biblepro to transfer the activation.",
            &claims.device_id
        ));
    }

    keystore::save_token(&token_str)?;
    keystore::clear_grace_start();

    let mut data = state.write();
    *data = LicenseData::from_claims(claims, token_str);
    data.device_id = machine;

    Ok(LicenseStatus::from(&*data))
}

// ── get_license_status ────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_license_status(state: State<'_, LicenseState>) -> LicenseStatus {
    LicenseStatus::from(&*state.read())
}

// ── deactivate_license ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn deactivate_license(state: State<'_, LicenseState>) -> Result<(), String> {
    keystore::clear_token();
    let machine = state.read().device_id.clone();
    let mut data = state.write();
    *data = LicenseData::default();
    data.device_id = machine;
    Ok(())
}

// ── refresh_license ───────────────────────────────────────────────────────────
// Called on app launch (and optionally on a timer) to silently renew the token
// while the machine is online. Falls back gracefully if Zehut is unreachable.

#[derive(Serialize)]
struct RefreshBody {
    org_id: String,
    device_id: String,
    current_token: String,
}

#[derive(Deserialize)]
struct RefreshResponse {
    token: String,
}

#[tauri::command]
pub async fn refresh_license(state: State<'_, LicenseState>) -> Result<LicenseStatus, String> {
    let (raw, org_id, device_id) = {
        let d = state.read();
        match d.raw_token.clone() {
            Some(t) => (t, d.org_id.clone(), d.device_id.clone()),
            None => return Ok(LicenseStatus::from(&*d)),
        }
    };

    // Check if still has > 3 days before expiry — skip refresh if so
    {
        let d = state.read();
        if let Some(exp) = d.expires_at {
            let days_left = (exp - now_unix()) / 86400;
            if days_left > 3 {
                return Ok(LicenseStatus::from(&*d));
            }
        }
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("{CLOUD_API}/v1/license/refresh");
    let resp = client
        .post(&url)
        .json(&RefreshBody {
            org_id,
            device_id,
            current_token: raw,
        })
        .send()
        .await
        .map_err(|e| format!("cloud unreachable: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("refresh rejected: HTTP {}", resp.status()));
    }

    let body: RefreshResponse = resp.json().await.map_err(|e| e.to_string())?;

    // Validate the freshly-issued token
    let machine = state.read().device_id.clone();
    let claims = token::verify_token(&body.token)?;
    if claims.device_id != machine {
        return Err("Refreshed token device_id mismatch".to_string());
    }

    keystore::save_token(&body.token)?;
    keystore::clear_grace_start();

    let mut data = state.write();
    *data = LicenseData::from_claims(claims, body.token);
    data.device_id = machine;

    Ok(LicenseStatus::from(&*data))
}
