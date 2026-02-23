use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Erreur de déchiffrement")]
    DecryptionFailed,
    #[error("Fichier clé introuvable")]
    KeyFileNotFound,
    #[error("Identifiants invalides")]
    InvalidCredentials,
    #[error("Saladier verrouillé")]
    SaladierLocked,
    #[error("Potager verrouillé")]
    PotagerLocked,
    #[error("Utilisateur déjà existant")]
    UserAlreadyExists,
    #[error("Utilisateur introuvable")]
    UserNotFound,
    #[error("Saladier introuvable")]
    SaladierNotFound,
    #[error("Feuille introuvable")]
    FeuilleNotFound,
    #[error("Erreur base de données : {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Erreur IO : {0}")]
    Io(#[from] std::io::Error),
    #[error("Session serveur expirée")]
    ServerUnauthorized,
    #[error("Code MFA invalide")]
    MfaInvalidCode,
    #[error("Configuration MFA expirée")]
    MfaSetupExpired,
    #[error("Erreur serveur : {0}")]
    ServerError(String),
    #[error("Erreur interne : {0}")]
    Internal(String),
}

// Tauri commands require Serialize for error types
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Security: return generic messages to the UI to avoid info leakage
        let generic_msg = match self {
            AppError::InvalidCredentials
            | AppError::UserNotFound
            | AppError::DecryptionFailed => "Identifiants invalides",
            AppError::KeyFileNotFound => "Fichier clé introuvable",
            AppError::SaladierLocked => "Saladier verrouillé",
            AppError::PotagerLocked => "Potager verrouillé",
            AppError::UserAlreadyExists => "Compte déjà existant",
            AppError::SaladierNotFound | AppError::FeuilleNotFound => "Ressource introuvable",
            AppError::ServerUnauthorized => "Session serveur expirée",
            AppError::MfaInvalidCode => "Code MFA invalide",
            AppError::MfaSetupExpired => "Configuration MFA expirée",
            // Server messages are already sanitized by the API — pass them through
            AppError::ServerError(msg) => return serializer.serialize_str(msg),
            AppError::Database(_) | AppError::Io(_) | AppError::Internal(_) => "Erreur interne",
        };
        serializer.serialize_str(generic_msg)
    }
}
