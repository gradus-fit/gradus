// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{error::Error, fmt};

use zeroize::{Zeroize, Zeroizing};

use crate::private;

/// A random personal vault key: public 16-byte routing ID plus a secret 32-byte key.
///
/// This type deliberately cannot be cloned, formatted, or serialized. Store only the
/// bytes from [`encode_personal_vault_key_for_storage`] in platform secure storage.
pub struct PersonalVaultKey {
    pub(crate) key_id: [u8; 16],
    pub(crate) secret: [u8; 32],
}

impl Drop for PersonalVaultKey {
    fn drop(&mut self) {
        self.key_id.zeroize();
        self.secret.zeroize();
    }
}

/// The immutable identity authenticated with an encrypted blob.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobBinding {
    pub data_space_id: [u8; 16],
    pub resource_type_id: [u8; 16],
    pub resource_id: [u8; 16],
    pub revision: u64,
}

/// A versioned, portable encrypted envelope.
///
/// Use [`encode_encrypted_blob`] for storage or transport and
/// [`decode_encrypted_blob`] for untrusted received bytes.
pub struct EncryptedBlob {
    pub(crate) payload_nonce: [u8; 24],
    pub(crate) payload_ciphertext: Vec<u8>,
    pub(crate) wrappers: Vec<Vec<u8>>,
}

/// An encryption, encoding, or authentication failure.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EncryptionError {
    InvalidKeyEncoding,
    MalformedEnvelope,
    UnsupportedEnvelopeVersion,
    UnsupportedCipherSuite,
    InputTooLarge,
    RandomnessUnavailable,
    AuthenticationFailed,
}

impl fmt::Display for EncryptionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidKeyEncoding => "invalid personal vault key encoding",
            Self::MalformedEnvelope => "malformed encrypted envelope",
            Self::UnsupportedEnvelopeVersion => "unsupported encrypted envelope version",
            Self::UnsupportedCipherSuite => "unsupported encrypted envelope cipher suite",
            Self::InputTooLarge => "encryption input is too large",
            Self::RandomnessUnavailable => "secure randomness is unavailable",
            Self::AuthenticationFailed => "encrypted blob authentication failed",
        };
        formatter.write_str(message)
    }
}

impl Error for EncryptionError {}

/// Generates a random personal vault key using the operating-system CSPRNG.
pub fn generate_personal_vault_key() -> Result<PersonalVaultKey, EncryptionError> {
    private::generate_personal_vault_key()
}

/// Decodes the canonical sensitive key representation retrieved from secure storage.
pub fn decode_personal_vault_key_from_storage(
    encoded: &[u8],
) -> Result<PersonalVaultKey, EncryptionError> {
    private::decode_personal_vault_key_from_storage(encoded)
}

/// Returns the canonical sensitive key representation for platform secure storage.
pub fn encode_personal_vault_key_for_storage(key: &PersonalVaultKey) -> Zeroizing<Vec<u8>> {
    private::encode_personal_vault_key_for_storage(key)
}

/// Decodes an untrusted portable encrypted envelope without decrypting it.
pub fn decode_encrypted_blob(encoded: &[u8]) -> Result<EncryptedBlob, EncryptionError> {
    private::decode_encrypted_blob(encoded)
}

/// Encodes an encrypted envelope in its canonical binary representation.
pub fn encode_encrypted_blob(blob: &EncryptedBlob) -> Vec<u8> {
    private::encode_encrypted_blob(blob)
}

/// Encrypts an in-memory blob and binds it to `binding`.
pub fn encrypt_blob(
    key: &PersonalVaultKey,
    plaintext: &[u8],
    binding: &BlobBinding,
) -> Result<EncryptedBlob, EncryptionError> {
    private::encrypt_blob(key, plaintext, binding)
}

/// Authenticates and decrypts an envelope that was bound to `binding`.
pub fn decrypt_blob(
    key: &PersonalVaultKey,
    encrypted: &EncryptedBlob,
    binding: &BlobBinding,
) -> Result<Zeroizing<Vec<u8>>, EncryptionError> {
    private::decrypt_blob(key, encrypted, binding)
}

#[cfg(test)]
mod tests {
    use super::EncryptionError;

    #[test]
    fn errors_have_fixed_non_sensitive_messages() {
        let cases = [
            (
                EncryptionError::InvalidKeyEncoding,
                "invalid personal vault key encoding",
            ),
            (
                EncryptionError::MalformedEnvelope,
                "malformed encrypted envelope",
            ),
            (
                EncryptionError::UnsupportedEnvelopeVersion,
                "unsupported encrypted envelope version",
            ),
            (
                EncryptionError::UnsupportedCipherSuite,
                "unsupported encrypted envelope cipher suite",
            ),
            (
                EncryptionError::InputTooLarge,
                "encryption input is too large",
            ),
            (
                EncryptionError::RandomnessUnavailable,
                "secure randomness is unavailable",
            ),
            (
                EncryptionError::AuthenticationFailed,
                "encrypted blob authentication failed",
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(error.to_string(), expected);
            assert!(std::error::Error::source(&error).is_none());
        }
    }
}
