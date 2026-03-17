use crate::i18n::{t, Language};

/// Validate a password against the security policy:
/// - Minimum 16 characters
/// - At least one uppercase letter
/// - At least one lowercase letter
/// - At least one digit
/// - At least one special character
pub fn validate_password(password: &str, lang: Language) -> Result<(), String> {
    if password.len() < 16 {
        return Err(t("pwd.min_length", lang).to_string());
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(t("pwd.uppercase", lang).to_string());
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(t("pwd.lowercase", lang).to_string());
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(t("pwd.digit", lang).to_string());
    }
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(t("pwd.special", lang).to_string());
    }
    Ok(())
}
