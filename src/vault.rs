use crate::models::Vault;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use rand::Rng;
use std::io;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

pub fn encrypt_vault(vault: Vault, password: &str) -> io::Result<Vec<u8>> {
    let json = serde_json::to_string(&vault)?;
    let salt_bytes: [u8; SALT_LEN] = rand::thread_rng().gen();
    let salt_string =
        SaltString::encode_b64(&salt_bytes).map_err(|e| io::Error::other(e.to_string()))?;

    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| io::Error::other(e.to_string()))?;

    // We need 32 bytes for AES-256
    let mut key_bytes = [0u8; 32];
    let derived_key = hash
        .hash
        .ok_or_else(|| io::Error::other("Failed to derive key"))?;
    // Argon2 might produce different lengths, so we copy what we need
    let len = derived_key.len().min(32);
    key_bytes[..len].copy_from_slice(&derived_key.as_bytes()[..len]);

    let cipher =
        Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| io::Error::other(e.to_string()))?;
    let nonce_bytes: [u8; NONCE_LEN] = rand::thread_rng().gen();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, json.as_bytes())
        .map_err(|e| io::Error::other(e.to_string()))?;

    let mut result = Vec::new();
    result.extend_from_slice(&salt_bytes);
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

pub fn decrypt_vault(data: &[u8], password: &str) -> io::Result<Vault> {
    if data.len() < SALT_LEN + NONCE_LEN {
        return Err(io::Error::other("Invalid vault file: too short"));
    }

    let salt_bytes = &data[..SALT_LEN];
    let nonce_bytes = &data[SALT_LEN..SALT_LEN + NONCE_LEN];
    let ciphertext = &data[SALT_LEN + NONCE_LEN..];

    let salt_string =
        SaltString::encode_b64(salt_bytes).map_err(|e| io::Error::other(e.to_string()))?;

    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| io::Error::other(e.to_string()))?;

    let mut key_bytes = [0u8; 32];
    let derived_key = hash
        .hash
        .ok_or_else(|| io::Error::other("Failed to derive key"))?;
    let len = derived_key.len().min(32);
    key_bytes[..len].copy_from_slice(&derived_key.as_bytes()[..len]);

    let cipher =
        Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| io::Error::other(e.to_string()))?;
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|_| {
        io::Error::other("Failed to decrypt vault: incorrect password or corrupted data")
    })?;

    let vault: Vault = serde_json::from_slice(&plaintext)?;
    Ok(vault)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Account, GityConfig, Platform, VaultKey};

    #[test]
    fn test_vault_encryption_cycle() {
        let config = GityConfig {
            accounts: vec![Account {
                name: "test".to_string(),
                platform: Platform::Github,
                key_path: "id_ed25519_test".to_string(),
                host_alias: "github-test".to_string(),
                username: "tester".to_string(),
                email: "test@example.com".to_string(),
            }],
        };

        let keys = vec![VaultKey {
            filename: "id_ed25519_test".to_string(),
            private_content: "PRIVATE KEY CONTENT".to_string(),
            public_content: "PUBLIC KEY CONTENT".to_string(),
        }];

        let vault = Vault { config, keys };
        let password = "strongpassword123";

        let encrypted = encrypt_vault(vault.clone(), password).expect("Encryption failed");
        let decrypted = decrypt_vault(&encrypted, password).expect("Decryption failed");

        assert_eq!(decrypted.config.accounts.len(), 1);
        assert_eq!(decrypted.keys.len(), 1);
        assert_eq!(decrypted.keys[0].private_content, "PRIVATE KEY CONTENT");
    }

    #[test]
    fn test_vault_wrong_password() {
        let vault = Vault {
            config: GityConfig::default(),
            keys: vec![],
        };
        let encrypted = encrypt_vault(vault, "password").unwrap();
        let result = decrypt_vault(&encrypted, "wrong_password");
        assert!(result.is_err());
    }
}
