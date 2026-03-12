use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;
use std::path::Path;
use zeroize::Zeroize;

use crate::crypto::argon2_kdf;
use crate::error::AppError;

/// Generate a new 32-byte device secret key using a CSPRNG.
pub fn generate_device_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

/// Save the device key to disk at the specified path.
/// On Unix, restricts file permissions to owner-only (0o600).
pub fn save_device_key(key: &[u8; 32], path: &Path) -> Result<(), AppError> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, key)?;

    // Restrict to owner-read/write only on Unix (Windows uses ACLs via %APPDATA%)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(path, perms)?;
    }

    Ok(())
}

/// Load the device key from disk. Returns KeyFileNotFound if missing.
pub fn load_device_key(path: &Path) -> Result<[u8; 32], AppError> {
    if !path.exists() {
        return Err(AppError::KeyFileNotFound);
    }

    let bytes = std::fs::read(path)?;
    if bytes.len() != 32 {
        return Err(AppError::Internal(
            "Device key file has invalid size".to_string(),
        ));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

/// Check if the device key file exists at the given path.
pub fn device_key_exists(path: &Path) -> bool {
    path.exists()
}

/// Reconstruct the Master Key using the Dual-Lock protocol with HKDF:
///   1. derived = Argon2id(master_password, salt)
///   2. PRK = HKDF-Extract(salt=device_key, ikm=derived)
///   3. Master_Key = HKDF-Expand(PRK, info="SaladVault_MasterKey_v2", len=32)
///
/// HKDF is the standard primitive for combining cryptographic key materials,
/// replacing the previous XOR approach with a more robust construction.
/// The result is zeroized when dropped.
pub fn reconstruct_master_key(
    master_password: &[u8],
    device_key: &[u8; 32],
    salt: &[u8],
) -> Result<MasterKey, AppError> {
    let mut derived = argon2_kdf::derive_key(master_password, salt)?;

    // HKDF-Extract(salt=device_key, ikm=argon2_output) then Expand
    let hk = Hkdf::<Sha256>::new(Some(device_key), &derived);
    let mut master_key = [0u8; 32];
    hk.expand(b"SaladVault_MasterKey_v2", &mut master_key)
        .map_err(|e| AppError::Internal(format!("HKDF error: {e}")))?;

    // Zeroize the intermediate derived key
    derived.zeroize();

    Ok(MasterKey { inner: master_key })
}

/// A wrapper around the 32-byte master key that zeroizes on drop.
pub struct MasterKey {
    inner: [u8; 32],
}

impl MasterKey {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.inner
    }
}

impl Drop for MasterKey {
    fn drop(&mut self) {
        self.inner.zeroize();
    }
}

impl Zeroize for MasterKey {
    fn zeroize(&mut self) {
        self.inner.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_generate_device_key_random() {
        let key1 = generate_device_key();
        let key2 = generate_device_key();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_reconstruct_master_key_deterministic() {
        let password = b"master_password";
        let device_key = [42u8; 32];
        let salt = [0u8; 32];

        let mk1 = reconstruct_master_key(password, &device_key, &salt).unwrap();
        let mk2 = reconstruct_master_key(password, &device_key, &salt).unwrap();

        assert_eq!(mk1.as_bytes(), mk2.as_bytes());
    }

    #[test]
    fn test_reconstruct_master_key_different_device_key() {
        let password = b"master_password";
        let device_key1 = [42u8; 32];
        let device_key2 = [99u8; 32];
        let salt = [0u8; 32];

        let mk1 = reconstruct_master_key(password, &device_key1, &salt).unwrap();
        let mk2 = reconstruct_master_key(password, &device_key2, &salt).unwrap();

        assert_ne!(mk1.as_bytes(), mk2.as_bytes());
    }
}
