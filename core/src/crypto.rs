//! Cryptographic utilities for Asgard Mail

use crate::error::{AsgardError, AsgardResult};
use sodiumoxide::crypto::secretbox;
use sodiumoxide::crypto::pwhash;
use std::collections::HashMap;

/// Encryption key for sensitive data
pub struct EncryptionKey {
    key: secretbox::Key,
}

impl EncryptionKey {
    /// Create a new encryption key from a password
    pub fn from_password(password: &str, salt: &[u8]) -> AsgardResult<Self> {
        let mut key_bytes = [0u8; secretbox::KEYBYTES];
        pwhash::derive_key(
            &mut key_bytes,
            password.as_bytes(),
            &pwhash::Salt::from_slice(salt).ok_or(AsgardError::crypto("Invalid salt"))?,
            pwhash::OPSLIMIT_INTERACTIVE,
            pwhash::MEMLIMIT_INTERACTIVE,
        ).map_err(|_| AsgardError::crypto("Failed to derive key"))?;
        
        Ok(Self {
            key: secretbox::Key::from_slice(&key_bytes).ok_or(AsgardError::crypto("Invalid key"))?,
        })
    }

    /// Create a new encryption key from random bytes
    pub fn new() -> Self {
        Self {
            key: secretbox::gen_key(),
        }
    }

    /// Get the key bytes
    pub fn as_bytes(&self) -> &[u8] {
        self.key.as_ref()
    }

    /// Encrypt data
    pub fn encrypt(&self, data: &[u8]) -> AsgardResult<Vec<u8>> {
        let nonce = secretbox::gen_nonce();
        let ciphertext = secretbox::seal(data, &nonce, &self.key);
        
        // Prepend nonce to ciphertext
        let mut result = Vec::with_capacity(nonce.as_ref().len() + ciphertext.len());
        result.extend_from_slice(nonce.as_ref());
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    /// Decrypt data
    pub fn decrypt(&self, encrypted_data: &[u8]) -> AsgardResult<Vec<u8>> {
        if encrypted_data.len() < secretbox::NONCEBYTES {
            return Err(AsgardError::crypto("Invalid encrypted data length"));
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(secretbox::NONCEBYTES);
        let nonce = secretbox::Nonce::from_slice(nonce_bytes)
            .ok_or(AsgardError::crypto("Invalid nonce"))?;

        secretbox::open(ciphertext, &nonce, &self.key)
            .map_err(|_| AsgardError::crypto("Decryption failed"))
    }
}

/// Secure storage for sensitive data
pub struct SecureStorage {
    key: EncryptionKey,
    data: HashMap<String, Vec<u8>>,
}

impl SecureStorage {
    /// Create a new secure storage instance
    pub fn new(key: EncryptionKey) -> Self {
        Self {
            key,
            data: HashMap::new(),
        }
    }

    /// Store encrypted data
    pub fn store(&mut self, key: &str, value: &[u8]) -> AsgardResult<()> {
        let encrypted = self.key.encrypt(value)?;
        self.data.insert(key.to_string(), encrypted);
        Ok(())
    }

    /// Retrieve and decrypt data
    pub fn retrieve(&self, key: &str) -> AsgardResult<Option<Vec<u8>>> {
        if let Some(encrypted) = self.data.get(key) {
            let decrypted = self.key.decrypt(encrypted)?;
            Ok(Some(decrypted))
        } else {
            Ok(None)
        }
    }

    /// Remove data
    pub fn remove(&mut self, key: &str) -> Option<Vec<u8>> {
        self.data.remove(key)
    }

    /// Check if key exists
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

/// Password hashing utilities
pub struct PasswordHasher;

impl PasswordHasher {
    /// Hash a password
    pub fn hash_password(password: &str) -> AsgardResult<String> {
        let salt = pwhash::gen_salt();
        let hash = pwhash::pwhash(
            password.as_bytes(),
            pwhash::OPSLIMIT_INTERACTIVE,
            pwhash::MEMLIMIT_INTERACTIVE,
        ).map_err(|_| AsgardError::crypto("Password hashing failed"))?;
        
        Ok(hex::encode(hash.as_ref()))
    }

    /// Verify a password against a hash
    pub fn verify_password(password: &str, hash: &str) -> AsgardResult<bool> {
        let hash_bytes = hex::decode(hash)
            .map_err(|_| AsgardError::crypto("Invalid hash format"))?;
        
        let hash = pwhash::HashedPassword::from_slice(&hash_bytes)
            .ok_or(AsgardError::crypto("Invalid hash"))?;
        
        Ok(pwhash::pwhash_verify(&hash, password.as_bytes()))
    }
}

/// Random number generation utilities
pub struct RandomGenerator;

impl RandomGenerator {
    /// Generate random bytes
    pub fn random_bytes(length: usize) -> Vec<u8> {
        use sodiumoxide::randombytes;
        randombytes::randombytes(length)
    }

    /// Generate a random string
    pub fn random_string(length: usize) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut result = String::with_capacity(length);
        
        for _ in 0..length {
            let idx = (sodiumoxide::randombytes::randombytes(1)[0] as usize) % CHARSET.len();
            result.push(CHARSET[idx] as char);
        }
        
        result
    }

    /// Generate a random UUID
    pub fn random_uuid() -> uuid::Uuid {
        uuid::Uuid::new_v4()
    }
}

/// Data integrity utilities
pub struct DataIntegrity;

impl DataIntegrity {
    /// Calculate SHA-256 hash
    pub fn sha256(data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    /// Calculate SHA-256 hash of a string
    pub fn sha256_string(data: &str) -> String {
        Self::sha256(data.as_bytes())
    }

    /// Verify data integrity
    pub fn verify(data: &[u8], expected_hash: &str) -> bool {
        let actual_hash = Self::sha256(data);
        actual_hash == expected_hash
    }
}

/// OAuth2 token encryption utilities
pub struct TokenEncryption {
    key: EncryptionKey,
}

impl TokenEncryption {
    /// Create a new token encryption instance
    pub fn new(key: EncryptionKey) -> Self {
        Self { key }
    }

    /// Encrypt an OAuth2 access token
    pub fn encrypt_access_token(&self, token: &str) -> AsgardResult<String> {
        let encrypted = self.key.encrypt(token.as_bytes())?;
        Ok(base64::encode(encrypted))
    }

    /// Decrypt an OAuth2 access token
    pub fn decrypt_access_token(&self, encrypted_token: &str) -> AsgardResult<String> {
        let encrypted_bytes = base64::decode(encrypted_token)
            .map_err(|_| AsgardError::crypto("Invalid base64 token"))?;
        let decrypted = self.key.decrypt(&encrypted_bytes)?;
        String::from_utf8(decrypted)
            .map_err(|_| AsgardError::crypto("Invalid UTF-8 token"))
    }

    /// Encrypt an OAuth2 refresh token
    pub fn encrypt_refresh_token(&self, token: &str) -> AsgardResult<String> {
        self.encrypt_access_token(token)
    }

    /// Decrypt an OAuth2 refresh token
    pub fn decrypt_refresh_token(&self, encrypted_token: &str) -> AsgardResult<String> {
        self.decrypt_access_token(encrypted_token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_key_creation() {
        let password = "test_password";
        let salt = RandomGenerator::random_bytes(32);
        
        let key = EncryptionKey::from_password(password, &salt).unwrap();
        assert_eq!(key.as_bytes().len(), secretbox::KEYBYTES);
    }

    #[test]
    fn test_encryption_decryption() {
        let key = EncryptionKey::new();
        let data = b"Hello, World!";
        
        let encrypted = key.encrypt(data).unwrap();
        let decrypted = key.decrypt(&encrypted).unwrap();
        
        assert_eq!(data, decrypted.as_slice());
    }

    #[test]
    fn test_secure_storage() {
        let key = EncryptionKey::new();
        let mut storage = SecureStorage::new(key);
        
        let test_data = b"secret data";
        storage.store("test_key", test_data).unwrap();
        
        let retrieved = storage.retrieve("test_key").unwrap().unwrap();
        assert_eq!(test_data, retrieved.as_slice());
        
        assert!(storage.contains_key("test_key"));
        assert!(!storage.contains_key("nonexistent_key"));
    }

    #[test]
    fn test_password_hashing() {
        let password = "test_password";
        let hash = PasswordHasher::hash_password(password).unwrap();
        
        assert!(PasswordHasher::verify_password(password, &hash).unwrap());
        assert!(!PasswordHasher::verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_random_generation() {
        let bytes = RandomGenerator::random_bytes(32);
        assert_eq!(bytes.len(), 32);
        
        let string = RandomGenerator::random_string(16);
        assert_eq!(string.len(), 16);
        
        let uuid = RandomGenerator::random_uuid();
        assert!(!uuid.is_nil());
    }

    #[test]
    fn test_data_integrity() {
        let data = b"test data";
        let hash = DataIntegrity::sha256(data);
        
        assert!(DataIntegrity::verify(data, &hash));
        assert!(!DataIntegrity::verify(b"different data", &hash));
    }

    #[test]
    fn test_token_encryption() {
        let key = EncryptionKey::new();
        let token_encryption = TokenEncryption::new(key);
        
        let access_token = "test_access_token";
        let encrypted = token_encryption.encrypt_access_token(access_token).unwrap();
        let decrypted = token_encryption.decrypt_access_token(&encrypted).unwrap();
        
        assert_eq!(access_token, decrypted);
    }
}
