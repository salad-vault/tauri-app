/// Validate a password against the security policy:
/// - Minimum 16 characters
/// - At least one uppercase letter
/// - At least one lowercase letter
/// - At least one digit
/// - At least one special character
pub fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 16 {
        return Err("Le mot de passe doit faire au moins 16 caractères.".to_string());
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err("Le mot de passe doit contenir au moins une majuscule.".to_string());
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err("Le mot de passe doit contenir au moins une minuscule.".to_string());
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err("Le mot de passe doit contenir au moins un chiffre.".to_string());
    }
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err("Le mot de passe doit contenir au moins un caractère spécial.".to_string());
    }
    Ok(())
}
