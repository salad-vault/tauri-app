use serde::{Deserialize, Serialize};

/// Auto-lock timeout options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AutoLockTimeout {
    Immediate,
    After1Min,
    After5Min,
    Never,
}

impl Default for AutoLockTimeout {
    fn default() -> Self {
        Self::After5Min
    }
}

/// Password generator type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PasswordType {
    Alphanumeric,
    Passphrase,
}

impl Default for PasswordType {
    fn default() -> Self {
        Self::Alphanumeric
    }
}

/// Favicon download policy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FaviconPolicy {
    None,
    ProxyAnonymous,
    Direct,
}

impl Default for FaviconPolicy {
    fn default() -> Self {
        Self::None
    }
}

/// Theme options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    #[serde(alias = "System")]
    Dark,
    Light,
}

impl Default for Theme {
    fn default() -> Self {
        Self::Dark
    }
}

/// User settings stored as JSON in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    // Security & Lock
    #[serde(default)]
    pub auto_lock_timeout: AutoLockTimeout,
    #[serde(default = "default_true")]
    pub auto_lock_on_sleep: bool,
    #[serde(default = "default_true")]
    pub auto_lock_on_close: bool,
    #[serde(default = "default_true")]
    pub auto_lock_on_inactivity: bool,
    #[serde(default = "default_clipboard_seconds")]
    pub clipboard_clear_seconds: u32,
    #[serde(default = "default_true")]
    pub screenshot_protection: bool,

    // Password generator defaults
    #[serde(default = "default_password_length")]
    pub password_default_length: u32,
    #[serde(default)]
    pub password_type: PasswordType,

    // Privacy
    #[serde(default)]
    pub favicon_policy: FaviconPolicy,
    #[serde(default)]
    pub crash_reports: bool,

    // Saladier
    #[serde(default)]
    pub max_failed_attempts: u32,

    // Theme
    #[serde(default)]
    pub theme: Theme,

    // Dead Man's Switch
    #[serde(default)]
    pub dead_man_switch_enabled: bool,
    #[serde(default = "default_dead_man_days")]
    pub dead_man_switch_days: u32,
    #[serde(default)]
    pub dead_man_switch_email: String,

    // General
    #[serde(default)]
    pub clear_icon_cache_on_close: bool,
}

fn default_true() -> bool {
    true
}

fn default_clipboard_seconds() -> u32 {
    30
}

fn default_password_length() -> u32 {
    20
}

fn default_dead_man_days() -> u32 {
    90
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            auto_lock_timeout: AutoLockTimeout::default(),
            auto_lock_on_sleep: true,
            auto_lock_on_close: true,
            auto_lock_on_inactivity: true,
            clipboard_clear_seconds: 30,
            screenshot_protection: true,
            password_default_length: 20,
            password_type: PasswordType::default(),
            favicon_policy: FaviconPolicy::default(),
            crash_reports: false,
            max_failed_attempts: 0,
            theme: Theme::default(),
            dead_man_switch_enabled: false,
            dead_man_switch_days: 90,
            dead_man_switch_email: String::new(),
            clear_icon_cache_on_close: false,
        }
    }
}
