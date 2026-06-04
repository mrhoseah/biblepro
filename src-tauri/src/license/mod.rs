pub mod commands;
pub mod device;
pub mod keystore;
pub mod token;

use std::sync::{Arc, RwLock};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

pub use token::{LicenseClaims, Plan};

// ── Cloud API ─────────────────────────────────────────────────────────────────

/// Zehut control-plane base URL.
/// Token refresh and device activation calls go here.
pub const CLOUD_API: &str = "https://api.zehut.io";

// ── Feature gates ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Feature {
    // ── Standard ──────────────────────────
    /// Advanced layout templates (LowerThirdAccent, LowerThirdSplit, CardCenter)
    AdvancedTemplates,
    /// Canvas gradient background / band designer
    CanvasDesign,
    /// 4K (3840×2160) NDI output
    Resolution4K,

    // ── Premium ───────────────────────────
    /// AI verse suggestions
    AiSuggestions,
    /// Cloud setlist sync
    CloudSync,
    /// Multiple simultaneous NDI outputs
    MultipleOutputs,
}

impl Feature {
    pub fn required_plan(&self) -> Plan {
        match self {
            Feature::AdvancedTemplates | Feature::CanvasDesign | Feature::Resolution4K
                => Plan::Standard,
            Feature::AiSuggestions | Feature::CloudSync | Feature::MultipleOutputs
                => Plan::Premium,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Feature::AdvancedTemplates => "Advanced Templates",
            Feature::CanvasDesign      => "Canvas Background Designer",
            Feature::Resolution4K      => "4K Output",
            Feature::AiSuggestions     => "AI Suggestions",
            Feature::CloudSync         => "Cloud Sync",
            Feature::MultipleOutputs   => "Multiple NDI Outputs",
        }
    }
}

// ── LicenseData ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LicenseData {
    /// Current effective plan (Free if no valid token)
    pub plan: Plan,
    /// Organisation name from token, empty on Free
    pub org: String,
    /// Organisation ID from token sub claim
    pub org_id: String,
    /// Machine fingerprint
    pub device_id: String,
    /// Raw JWT stored for refresh calls
    pub raw_token: Option<String>,
    /// Token expiry (Unix seconds), None = never (dev override)
    pub expires_at: Option<i64>,
    /// When the grace period started (None = not in grace)
    pub grace_started_at: Option<i64>,
}

impl Default for LicenseData {
    fn default() -> Self {
        Self {
            plan: Plan::Free,
            org: String::new(),
            org_id: String::new(),
            device_id: String::new(),
            raw_token: None,
            expires_at: None,
            grace_started_at: None,
        }
    }
}

impl LicenseData {
    pub fn from_claims(claims: LicenseClaims, raw: String) -> Self {
        Self {
            plan: claims.plan,
            org: claims.org,
            org_id: claims.sub,
            device_id: claims.device_id,
            raw_token: Some(raw),
            expires_at: Some(claims.exp),
            grace_started_at: None,
        }
    }

    pub fn is_in_grace(&self) -> bool {
        self.grace_started_at.is_some()
    }

    pub fn grace_days_remaining(&self) -> Option<i64> {
        let started = self.grace_started_at?;
        let now = now_unix();
        let elapsed_days = (now - started) / 86400;
        let remaining = token::GRACE_DAYS - elapsed_days;
        Some(remaining.max(0))
    }
}

// ── LicenseState ─────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct LicenseState {
    inner: Arc<RwLock<LicenseData>>,
    pub app_data_dir: PathBuf,
}

impl LicenseState {
    pub fn new(app_data_dir: PathBuf) -> Self {
        Self {
            inner: Arc::new(RwLock::new(LicenseData::default())),
            app_data_dir,
        }
    }

    pub fn read(&self) -> std::sync::RwLockReadGuard<'_, LicenseData> {
        self.inner.read().unwrap()
    }

    pub fn write(&self) -> std::sync::RwLockWriteGuard<'_, LicenseData> {
        self.inner.write().unwrap()
    }

    /// Load and validate any previously saved token from the OS keychain.
    /// Called once at startup. Never panics — falls back to Free.
    pub fn init(&self) {
        let machine = device::machine_id(&self.app_data_dir);
        self.write().device_id = machine.clone();

        let Some(raw) = keystore::load_token() else { return };

        match token::verify_token(&raw) {
            Ok(claims) if claims.device_id == machine => {
                let mut d = self.write();
                *d = LicenseData::from_claims(claims, raw);
                d.device_id = machine;
                // Clear any leftover grace marker on successful verification
                keystore::clear_grace_start();
            }
            Err(_) => {
                // Token is expired or signature invalid — try lenient decode for grace
                if let Ok(claims) = token::verify_token_lenient(&raw) {
                    if claims.device_id == machine {
                        let grace_start = keystore::load_grace_start().unwrap_or_else(|| {
                            let now = now_unix();
                            let _ = keystore::save_grace_start(now);
                            now
                        });
                        let elapsed = (now_unix() - grace_start) / 86400;
                        if elapsed <= token::GRACE_DAYS {
                            let mut d = self.write();
                            *d = LicenseData::from_claims(claims, raw);
                            d.device_id = machine;
                            d.grace_started_at = Some(grace_start);
                        } else {
                            // Grace expired — revert to Free, clear storage
                            keystore::clear_token();
                        }
                    }
                } else {
                    keystore::clear_token();
                }
            }
            _ => {
                // Device ID mismatch — do not apply token
                keystore::clear_token();
            }
        }
    }

    /// Check whether the current session has a given feature enabled.
    pub fn has_feature(&self, feature: &Feature) -> bool {
        self.read().plan.includes(&feature.required_plan())
    }

    /// Return `Ok(())` if the feature is available, `Err(message)` otherwise.
    pub fn require_feature(&self, feature: Feature) -> Result<(), String> {
        if self.has_feature(&feature) {
            Ok(())
        } else {
            Err(format!(
                "{} requires the {} plan — upgrade at zehut.io/biblepro",
                feature.display_name(),
                feature.required_plan().label()
            ))
        }
    }
}

// ── Serializable status (sent to frontend) ────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LicenseStatus {
    pub plan: Plan,
    pub org: String,
    pub org_id: String,
    pub device_id: String,
    pub expires_at: Option<i64>,
    pub is_in_grace: bool,
    pub grace_days_remaining: Option<i64>,
    pub is_active: bool,
}

impl From<&LicenseData> for LicenseStatus {
    fn from(d: &LicenseData) -> Self {
        let is_active = d.plan != Plan::Free || d.raw_token.is_some();
        Self {
            plan: d.plan.clone(),
            org: d.org.clone(),
            org_id: d.org_id.clone(),
            device_id: d.device_id.clone(),
            expires_at: d.expires_at,
            is_in_grace: d.is_in_grace(),
            grace_days_remaining: d.grace_days_remaining(),
            is_active,
        }
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

pub fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
