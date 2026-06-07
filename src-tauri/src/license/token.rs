use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

// Generated once; private key lives on the Zehut backend only.
// Replace before production build.
pub const PUBLIC_KEY_PEM: &str = "-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAENXBr3alqXj4H+y2RQaPHTwmSDVNI
44B6pA0nJPo/ZRUPE1z80poORvTdjHoVEVyZtMpbkrCFZNZMk2insfLwAw==
-----END PUBLIC KEY-----";

pub const ISSUER: &str = "biblepro-zehut";

// Offline grace window: how long the app stays at current plan
// after failing to reach the cloud for a renewal.
pub const GRACE_DAYS: i64 = 14;

// ── Plan ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Plan {
    Free,
    Standard,
    Premium,
}

impl Plan {
    pub fn includes(&self, required: &Plan) -> bool {
        self >= required
    }

    pub fn label(&self) -> &'static str {
        match self {
            Plan::Free => "Free",
            Plan::Standard => "Standard",
            Plan::Premium => "Premium",
        }
    }
}

impl Default for Plan {
    fn default() -> Self {
        Plan::Free
    }
}

// ── JWT claims ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LicenseClaims {
    /// Organization identifier (stable across renewals)
    pub sub: String,
    /// Display name of the organisation / church
    pub org: String,
    pub plan: Plan,
    /// Maximum concurrent activated devices
    pub max_devices: u32,
    /// Machine ID this token was issued for
    pub device_id: String,
    /// Issued-at (Unix seconds)
    pub iat: i64,
    /// Expiry (Unix seconds)
    pub exp: i64,
    pub iss: String,
}

// ── Verification ──────────────────────────────────────────────────────────────

pub fn verify_token(token: &str) -> Result<LicenseClaims, String> {
    let key = DecodingKey::from_ec_pem(PUBLIC_KEY_PEM.as_bytes())
        .map_err(|e| format!("embedded public key invalid: {e}"))?;

    let mut validation = Validation::new(Algorithm::ES256);
    validation.set_issuer(&[ISSUER]);

    decode::<LicenseClaims>(token, &key, &validation)
        .map(|d| d.claims)
        .map_err(|e| format!("token verification failed: {e}"))
}

/// Verify without checking expiry — used when deciding whether to enter grace.
pub fn verify_token_lenient(token: &str) -> Result<LicenseClaims, String> {
    let key = DecodingKey::from_ec_pem(PUBLIC_KEY_PEM.as_bytes())
        .map_err(|e| format!("embedded public key invalid: {e}"))?;

    let mut validation = Validation::new(Algorithm::ES256);
    validation.set_issuer(&[ISSUER]);
    validation.validate_exp = false;

    decode::<LicenseClaims>(token, &key, &validation)
        .map(|d| d.claims)
        .map_err(|e| format!("token verification failed: {e}"))
}
