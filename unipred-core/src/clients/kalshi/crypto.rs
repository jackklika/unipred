use base64::{Engine as _, engine::general_purpose};
use rsa::{RsaPrivateKey, pss::BlindedSigningKey, signature::{RandomizedSigner, SignatureEncoding}};
use sha2::Sha256;
use rand::rngs::OsRng;

// Error type for signing operations
#[derive(Debug)]
pub enum SignError {
    SigningFailed(String),
}

impl std::fmt::Display for SignError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignError::SigningFailed(msg) => write!(f, "RSA sign PSS failed: {}", msg),
        }
    }
}

impl std::error::Error for SignError {}

/// Sign text using RSA-PSS with SHA-256
pub fn sign_pss_text(private_key: &RsaPrivateKey, text: &str) -> Result<String, SignError> {
    let message = text.as_bytes();

    // Create PSS signing key with SHA-256
    let signing_key = BlindedSigningKey::<Sha256>::new(private_key.clone());

    // Sign the message
    let signature = signing_key
        .sign_with_rng(&mut OsRng, message)
        .to_bytes();

    // Encode signature as base64
    Ok(general_purpose::STANDARD.encode(&signature))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::RsaPrivateKey;
    use rand::rngs::OsRng;
    use std::sync::LazyLock;

    static TEST_KEY: LazyLock<RsaPrivateKey> = LazyLock::new(|| {
        RsaPrivateKey::new(&mut OsRng, 2048).unwrap()
    });

    fn get_test_key() -> &'static RsaPrivateKey {
        &TEST_KEY
    }

    #[test]
    fn test_sign_pss_text() {
        let private_key = get_test_key();

        let text = "Hello, World!";
        let signature = sign_pss_text(private_key, text).unwrap();

        // Signature should be valid base64
        assert!(general_purpose::STANDARD.decode(&signature).is_ok());

        // Basic length check - PSS signatures for 2048-bit keys should be 256 bytes
        let sig_bytes = general_purpose::STANDARD.decode(&signature).unwrap();
        assert_eq!(sig_bytes.len(), 256);
    }

    #[test]
    fn test_sign_pss_text_different_messages() {
        let private_key = get_test_key();

        let text1 = "Hello, World!";
        let text2 = "Goodbye, World!";
        
        let signature1 = sign_pss_text(private_key, text1).unwrap();
        let signature2 = sign_pss_text(private_key, text2).unwrap();

        // Different messages should produce different signatures
        assert_ne!(signature1, signature2);
    }
}