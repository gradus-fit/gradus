// SPDX-License-Identifier: AGPL-3.0-or-later

use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit, Payload},
};
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::Zeroizing;

use crate::api::{BlobBinding, EncryptedBlob, EncryptionError, PersonalVaultKey};

const ENVELOPE_PREFIX: &[u8; 7] = b"GRDENC\0";
const ENVELOPE_VERSION: u8 = 1;
const PAYLOAD_SUITE: u8 = 1;
const KEY_PREFIX: &[u8; 8] = b"GRDVKEY\0";
const KEY_VERSION: u8 = 1;
const KEY_ENCODED_LENGTH: usize = 57;
const NONCE_LENGTH: usize = 24;
const TAG_LENGTH: usize = 16;
const BLOB_KEY_LENGTH: usize = 32;
const WRAPPED_BLOB_KEY_LENGTH: usize = BLOB_KEY_LENGTH + TAG_LENGTH;
const MAX_PLAINTEXT_LENGTH: usize = 16 * 1024 * 1024;
const MAX_ENVELOPE_LENGTH: usize = MAX_PLAINTEXT_LENGTH + 4 * 1024;
const MAX_WRAPPER_COUNT: usize = 16;
const WRAPPER_TYPE_SYMMETRIC_VAULT: u8 = 1;
const WRAPPER_VERSION_SYMMETRIC_VAULT: u8 = 1;
const PAYLOAD_AAD_PREFIX: &[u8] = b"gradus/encryption/v1/payload\0";
const WRAPPER_AAD_PREFIX: &[u8] = b"gradus/encryption/v1/key-wrap\0";
const WRAPPING_KEY_INFO: &[u8] = b"gradus/encryption/v1/symmetric-blob-key-wrap";

pub(crate) fn generate_personal_vault_key() -> Result<PersonalVaultKey, EncryptionError> {
    let mut key_id = [0_u8; 16];
    let mut secret = [0_u8; 32];
    fill_random(&mut key_id)?;
    fill_random(&mut secret)?;
    Ok(PersonalVaultKey { key_id, secret })
}

pub(crate) fn decode_personal_vault_key_from_storage(
    encoded: &[u8],
) -> Result<PersonalVaultKey, EncryptionError> {
    if encoded.len() != KEY_ENCODED_LENGTH
        || encoded[..8] != *KEY_PREFIX
        || encoded[8] != KEY_VERSION
    {
        return Err(EncryptionError::InvalidKeyEncoding);
    }

    let mut key_id = [0_u8; 16];
    let mut secret = [0_u8; 32];
    key_id.copy_from_slice(&encoded[9..25]);
    secret.copy_from_slice(&encoded[25..57]);
    Ok(PersonalVaultKey { key_id, secret })
}

pub(crate) fn encode_personal_vault_key_for_storage(key: &PersonalVaultKey) -> Zeroizing<Vec<u8>> {
    let mut encoded = Zeroizing::new(Vec::with_capacity(KEY_ENCODED_LENGTH));
    encoded.extend_from_slice(KEY_PREFIX);
    encoded.push(KEY_VERSION);
    encoded.extend_from_slice(&key.key_id);
    encoded.extend_from_slice(&key.secret);
    encoded
}

pub(crate) fn decode_encrypted_blob(encoded: &[u8]) -> Result<EncryptedBlob, EncryptionError> {
    if encoded.len() > MAX_ENVELOPE_LENGTH {
        return Err(EncryptionError::InputTooLarge);
    }

    let mut cursor = encoded;
    let prefix = take(&mut cursor, ENVELOPE_PREFIX.len())?;
    if prefix != ENVELOPE_PREFIX {
        return Err(EncryptionError::MalformedEnvelope);
    }
    if take_u8(&mut cursor)? != ENVELOPE_VERSION {
        return Err(EncryptionError::UnsupportedEnvelopeVersion);
    }
    if take_u8(&mut cursor)? != PAYLOAD_SUITE {
        return Err(EncryptionError::UnsupportedCipherSuite);
    }

    let payload_nonce = take_array::<NONCE_LENGTH>(&mut cursor)?;
    let payload_ciphertext_length = usize::try_from(take_u32_be(&mut cursor)?)
        .map_err(|_| EncryptionError::MalformedEnvelope)?;
    if payload_ciphertext_length > MAX_PLAINTEXT_LENGTH + TAG_LENGTH {
        return Err(EncryptionError::InputTooLarge);
    }
    if payload_ciphertext_length < TAG_LENGTH {
        return Err(EncryptionError::MalformedEnvelope);
    }
    let payload_ciphertext = take(&mut cursor, payload_ciphertext_length)?.to_vec();

    let wrapper_count = usize::from(take_u8(&mut cursor)?);
    if !(1..=MAX_WRAPPER_COUNT).contains(&wrapper_count) {
        return Err(EncryptionError::MalformedEnvelope);
    }

    let mut wrappers = Vec::with_capacity(wrapper_count);
    for _ in 0..wrapper_count {
        let encoded_wrapper = take_wrapper(&mut cursor)?;
        validate_known_wrapper(encoded_wrapper)?;
        wrappers.push(encoded_wrapper.to_vec());
    }
    if !cursor.is_empty() {
        return Err(EncryptionError::MalformedEnvelope);
    }

    Ok(EncryptedBlob {
        payload_nonce,
        payload_ciphertext,
        wrappers,
    })
}

pub(crate) fn encode_encrypted_blob(blob: &EncryptedBlob) -> Vec<u8> {
    let wrapper_length: usize = blob.wrappers.iter().map(Vec::len).sum();
    let mut encoded = Vec::with_capacity(
        ENVELOPE_PREFIX.len()
            + 1
            + 1
            + NONCE_LENGTH
            + 4
            + blob.payload_ciphertext.len()
            + 1
            + wrapper_length,
    );
    encoded.extend_from_slice(ENVELOPE_PREFIX);
    encoded.push(ENVELOPE_VERSION);
    encoded.push(PAYLOAD_SUITE);
    encoded.extend_from_slice(&blob.payload_nonce);
    encoded.extend_from_slice(&(blob.payload_ciphertext.len() as u32).to_be_bytes());
    encoded.extend_from_slice(&blob.payload_ciphertext);
    encoded.push(blob.wrappers.len() as u8);
    for wrapper in &blob.wrappers {
        encoded.extend_from_slice(wrapper);
    }
    encoded
}

pub(crate) fn encrypt_blob(
    key: &PersonalVaultKey,
    plaintext: &[u8],
    binding: &BlobBinding,
) -> Result<EncryptedBlob, EncryptionError> {
    encrypt_blob_with_randomness(key, plaintext, binding, fill_random)
}

pub(crate) fn decrypt_blob(
    key: &PersonalVaultKey,
    encrypted: &EncryptedBlob,
    binding: &BlobBinding,
) -> Result<Zeroizing<Vec<u8>>, EncryptionError> {
    let payload_aad = payload_aad(binding);
    let wrapping_key = derive_wrapping_key(key);

    for encoded_wrapper in &encrypted.wrappers {
        let wrapper = match parse_wrapper(encoded_wrapper) {
            Ok(wrapper) => wrapper,
            Err(_) => return Err(EncryptionError::AuthenticationFailed),
        };
        if wrapper.wrapper_type != WRAPPER_TYPE_SYMMETRIC_VAULT
            || wrapper.version != WRAPPER_VERSION_SYMMETRIC_VAULT
            || wrapper.wrapping_key_id != key.key_id
        {
            continue;
        }

        let wrapper_aad = wrapper_aad(
            wrapper.wrapper_type,
            wrapper.version,
            &wrapper.wrapping_key_id,
            binding,
        );
        let wrapping_cipher = XChaCha20Poly1305::new((&*wrapping_key).into());
        let blob_key = match wrapping_cipher.decrypt(
            XNonce::from_slice(&wrapper.nonce),
            Payload {
                msg: wrapper.ciphertext,
                aad: &wrapper_aad,
            },
        ) {
            Ok(blob_key) if blob_key.len() == BLOB_KEY_LENGTH => Zeroizing::new(blob_key),
            _ => return Err(EncryptionError::AuthenticationFailed),
        };

        let payload_cipher = XChaCha20Poly1305::new(blob_key.as_slice().into());
        let plaintext = match payload_cipher.decrypt(
            XNonce::from_slice(&encrypted.payload_nonce),
            Payload {
                msg: &encrypted.payload_ciphertext,
                aad: &payload_aad,
            },
        ) {
            Ok(plaintext) => Zeroizing::new(plaintext),
            Err(_) => return Err(EncryptionError::AuthenticationFailed),
        };
        return Ok(plaintext);
    }

    Err(EncryptionError::AuthenticationFailed)
}

fn encrypt_blob_with_randomness<F>(
    key: &PersonalVaultKey,
    plaintext: &[u8],
    binding: &BlobBinding,
    mut random: F,
) -> Result<EncryptedBlob, EncryptionError>
where
    F: FnMut(&mut [u8]) -> Result<(), EncryptionError>,
{
    if plaintext.len() > MAX_PLAINTEXT_LENGTH {
        return Err(EncryptionError::InputTooLarge);
    }

    let mut blob_key = Zeroizing::new([0_u8; BLOB_KEY_LENGTH]);
    let mut payload_nonce = [0_u8; NONCE_LENGTH];
    let mut wrapper_nonce = [0_u8; NONCE_LENGTH];
    random(blob_key.as_mut())?;
    random(&mut payload_nonce)?;
    random(&mut wrapper_nonce)?;

    let payload_aad = payload_aad(binding);
    let payload_cipher = XChaCha20Poly1305::new(blob_key.as_slice().into());
    let payload_ciphertext = payload_cipher
        .encrypt(
            XNonce::from_slice(&payload_nonce),
            Payload {
                msg: plaintext,
                aad: &payload_aad,
            },
        )
        .map_err(|_| EncryptionError::AuthenticationFailed)?;

    let wrapping_key = derive_wrapping_key(key);
    let wrapper_aad = wrapper_aad(
        WRAPPER_TYPE_SYMMETRIC_VAULT,
        WRAPPER_VERSION_SYMMETRIC_VAULT,
        &key.key_id,
        binding,
    );
    let wrapping_cipher = XChaCha20Poly1305::new((&*wrapping_key).into());
    let wrapped_key_ciphertext = wrapping_cipher
        .encrypt(
            XNonce::from_slice(&wrapper_nonce),
            Payload {
                msg: blob_key.as_slice(),
                aad: &wrapper_aad,
            },
        )
        .map_err(|_| EncryptionError::AuthenticationFailed)?;

    let mut wrapper = Vec::with_capacity(2 + 16 + NONCE_LENGTH + 2 + WRAPPED_BLOB_KEY_LENGTH);
    wrapper.push(WRAPPER_TYPE_SYMMETRIC_VAULT);
    wrapper.push(WRAPPER_VERSION_SYMMETRIC_VAULT);
    wrapper.extend_from_slice(&key.key_id);
    wrapper.extend_from_slice(&wrapper_nonce);
    wrapper.extend_from_slice(&(wrapped_key_ciphertext.len() as u16).to_be_bytes());
    wrapper.extend_from_slice(&wrapped_key_ciphertext);

    Ok(EncryptedBlob {
        payload_nonce,
        payload_ciphertext,
        wrappers: vec![wrapper],
    })
}

fn fill_random(destination: &mut [u8]) -> Result<(), EncryptionError> {
    getrandom::fill(destination).map_err(|_| EncryptionError::RandomnessUnavailable)
}

fn derive_wrapping_key(key: &PersonalVaultKey) -> Zeroizing<[u8; 32]> {
    let hkdf = Hkdf::<Sha256>::new(None, &key.secret);
    let mut wrapping_key = Zeroizing::new([0_u8; 32]);
    hkdf.expand(WRAPPING_KEY_INFO, wrapping_key.as_mut())
        .expect("fixed HKDF output length is valid");
    wrapping_key
}

fn payload_aad(binding: &BlobBinding) -> Vec<u8> {
    let mut aad = Vec::with_capacity(PAYLOAD_AAD_PREFIX.len() + 56);
    aad.extend_from_slice(PAYLOAD_AAD_PREFIX);
    append_binding(&mut aad, binding);
    aad
}

fn wrapper_aad(
    wrapper_type: u8,
    wrapper_version: u8,
    wrapping_key_id: &[u8; 16],
    binding: &BlobBinding,
) -> Vec<u8> {
    let mut aad = Vec::with_capacity(WRAPPER_AAD_PREFIX.len() + 2 + 16 + 56);
    aad.extend_from_slice(WRAPPER_AAD_PREFIX);
    aad.push(wrapper_type);
    aad.push(wrapper_version);
    aad.extend_from_slice(wrapping_key_id);
    append_binding(&mut aad, binding);
    aad
}

fn append_binding(destination: &mut Vec<u8>, binding: &BlobBinding) {
    destination.extend_from_slice(&binding.data_space_id);
    destination.extend_from_slice(&binding.resource_type_id);
    destination.extend_from_slice(&binding.resource_id);
    destination.extend_from_slice(&binding.revision.to_be_bytes());
}

struct ParsedWrapper<'a> {
    wrapper_type: u8,
    version: u8,
    wrapping_key_id: [u8; 16],
    nonce: [u8; NONCE_LENGTH],
    ciphertext: &'a [u8],
}

fn take_wrapper<'a>(cursor: &mut &'a [u8]) -> Result<&'a [u8], EncryptionError> {
    let start = *cursor;
    let _ = take_u8(cursor)?;
    let _ = take_u8(cursor)?;
    let _ = take(cursor, 16)?;
    let _ = take(cursor, NONCE_LENGTH)?;
    let ciphertext_length = usize::from(take_u16_be(cursor)?);
    let _ = take(cursor, ciphertext_length)?;
    let consumed = start.len() - cursor.len();
    Ok(&start[..consumed])
}

fn validate_known_wrapper(encoded_wrapper: &[u8]) -> Result<(), EncryptionError> {
    let wrapper = parse_wrapper(encoded_wrapper)?;
    if wrapper.wrapper_type == WRAPPER_TYPE_SYMMETRIC_VAULT
        && wrapper.version == WRAPPER_VERSION_SYMMETRIC_VAULT
        && wrapper.ciphertext.len() != WRAPPED_BLOB_KEY_LENGTH
    {
        return Err(EncryptionError::MalformedEnvelope);
    }
    Ok(())
}

fn parse_wrapper(encoded_wrapper: &[u8]) -> Result<ParsedWrapper<'_>, EncryptionError> {
    let mut cursor = encoded_wrapper;
    let wrapper_type = take_u8(&mut cursor)?;
    let version = take_u8(&mut cursor)?;
    let wrapping_key_id = take_array::<16>(&mut cursor)?;
    let nonce = take_array::<NONCE_LENGTH>(&mut cursor)?;
    let ciphertext_length = usize::from(take_u16_be(&mut cursor)?);
    let ciphertext = take(&mut cursor, ciphertext_length)?;
    if !cursor.is_empty() {
        return Err(EncryptionError::MalformedEnvelope);
    }
    Ok(ParsedWrapper {
        wrapper_type,
        version,
        wrapping_key_id,
        nonce,
        ciphertext,
    })
}

fn take<'a>(cursor: &mut &'a [u8], length: usize) -> Result<&'a [u8], EncryptionError> {
    if cursor.len() < length {
        return Err(EncryptionError::MalformedEnvelope);
    }
    let (taken, remainder) = cursor.split_at(length);
    *cursor = remainder;
    Ok(taken)
}

fn take_array<const LENGTH: usize>(cursor: &mut &[u8]) -> Result<[u8; LENGTH], EncryptionError> {
    let mut array = [0_u8; LENGTH];
    array.copy_from_slice(take(cursor, LENGTH)?);
    Ok(array)
}

fn take_u8(cursor: &mut &[u8]) -> Result<u8, EncryptionError> {
    Ok(take(cursor, 1)?[0])
}

fn take_u16_be(cursor: &mut &[u8]) -> Result<u16, EncryptionError> {
    Ok(u16::from_be_bytes(take_array::<2>(cursor)?))
}

fn take_u32_be(cursor: &mut &[u8]) -> Result<u32, EncryptionError> {
    Ok(u32::from_be_bytes(take_array::<4>(cursor)?))
}

#[cfg(test)]
pub(crate) fn encrypt_blob_with_test_randomness(
    key: &PersonalVaultKey,
    plaintext: &[u8],
    binding: &BlobBinding,
    random_bytes: &[u8],
) -> Result<EncryptedBlob, EncryptionError> {
    let mut remaining = random_bytes;
    encrypt_blob_with_randomness(key, plaintext, binding, |destination| {
        if remaining.len() < destination.len() {
            return Err(EncryptionError::RandomnessUnavailable);
        }
        let (source, next) = remaining.split_at(destination.len());
        destination.copy_from_slice(source);
        remaining = next;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::{
        ENVELOPE_PREFIX, ENVELOPE_VERSION, EncryptionError, KEY_PREFIX, KEY_VERSION,
        MAX_ENVELOPE_LENGTH, MAX_PLAINTEXT_LENGTH, PAYLOAD_SUITE, decode_encrypted_blob,
        decode_personal_vault_key_from_storage, encode_encrypted_blob,
        encode_personal_vault_key_for_storage, encrypt_blob_with_test_randomness,
    };
    use crate::api::{BlobBinding, PersonalVaultKey};

    fn key() -> PersonalVaultKey {
        PersonalVaultKey {
            key_id: [0xA1; 16],
            secret: [0xB2; 32],
        }
    }

    fn binding() -> BlobBinding {
        BlobBinding {
            data_space_id: [
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
                0x0E, 0x0F,
            ],
            resource_type_id: [
                0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D,
                0x1E, 0x1F,
            ],
            resource_id: [
                0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D,
                0x8E, 0x8F,
            ],
            revision: 0x0102_0304_0506_0708,
        }
    }

    #[test]
    fn deterministic_encryption_vector_is_stable() {
        let random: Vec<u8> = (0..80).collect();
        let blob =
            encrypt_blob_with_test_randomness(&key(), b"vector plaintext", &binding(), &random)
                .expect("deterministic randomness succeeds");
        let encoded = encode_encrypted_blob(&blob);

        assert_eq!(
            encoded,
            hex::decode("475244454e43000101202122232425262728292a2b2c2d2e2f3031323334353637000000206b3c2ebf144534fb54df39d33c2279a51a95d051d4fa5bd7673ecb96588f7065010101a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a138393a3b3c3d3e3f404142434445464748494a4b4c4d4e4f0030ca396f3cc639003a836e45bb8a8533ecc72685d644168fe50732d3f7a4c3401100464ac1089b7a0eb1075952ddcbf86c")
                .expect("valid vector hex")
        );
    }

    #[test]
    fn deterministic_randomness_requires_enough_bytes() {
        assert!(matches!(
            encrypt_blob_with_test_randomness(&key(), b"", &binding(), &[]),
            Err(EncryptionError::RandomnessUnavailable)
        ));
    }

    #[test]
    fn key_and_envelope_decoders_reject_invalid_fixed_fields() {
        let mut encoded_key = encode_personal_vault_key_for_storage(&key()).to_vec();
        encoded_key[0] ^= 1;
        assert!(matches!(
            decode_personal_vault_key_from_storage(&encoded_key),
            Err(EncryptionError::InvalidKeyEncoding)
        ));
        assert_eq!(KEY_PREFIX, b"GRDVKEY\0");
        assert_eq!(KEY_VERSION, 1);

        let mut envelope = vec![0; MAX_ENVELOPE_LENGTH + 1];
        assert!(matches!(
            decode_encrypted_blob(&envelope),
            Err(EncryptionError::InputTooLarge)
        ));
        envelope.truncate(8);
        envelope[..7].copy_from_slice(ENVELOPE_PREFIX);
        envelope[7] = ENVELOPE_VERSION + 1;
        assert!(matches!(
            decode_encrypted_blob(&envelope),
            Err(EncryptionError::UnsupportedEnvelopeVersion)
        ));
        envelope[7] = ENVELOPE_VERSION;
        assert!(matches!(
            decode_encrypted_blob(&envelope),
            Err(EncryptionError::MalformedEnvelope)
        ));
        envelope.push(PAYLOAD_SUITE + 1);
        assert!(matches!(
            decode_encrypted_blob(&envelope),
            Err(EncryptionError::UnsupportedCipherSuite)
        ));
        assert!(MAX_PLAINTEXT_LENGTH > 0);
    }
}
