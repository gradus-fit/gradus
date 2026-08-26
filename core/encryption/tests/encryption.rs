// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::BTreeSet;

use gradus_encryption::{
    BlobBinding, EncryptionError, decode_encrypted_blob, decode_personal_vault_key_from_storage,
    decrypt_blob, encode_encrypted_blob, encode_personal_vault_key_for_storage, encrypt_blob,
    generate_personal_vault_key,
};

const VECTOR_KEY: &str = "475244564b45590001a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2";
const VECTOR_ENVELOPE: &str = "475244454e43000101202122232425262728292a2b2c2d2e2f3031323334353637000000206b3c2ebf144534fb54df39d33c2279a51a95d051d4fa5bd7673ecb96588f7065010101a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a138393a3b3c3d3e3f404142434445464748494a4b4c4d4e4f0030ca396f3cc639003a836e45bb8a8533ecc72685d644168fe50732d3f7a4c3401100464ac1089b7a0eb1075952ddcbf86c";
const VECTOR_UNKNOWN_WRAPPER: &str =
    "6307444444444444444444444444444444441010101010101010101010101010101010101010101010100002dead";

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

fn error<T>(result: Result<T, EncryptionError>) -> EncryptionError {
    result.err().expect("operation should fail")
}

fn decode_hex(value: &str) -> Vec<u8> {
    hex::decode(value).expect("test vector is valid hex")
}

#[test]
fn key_generation_storage_encoding_and_blob_round_trip_work() {
    let mut encodings = BTreeSet::new();
    for _ in 0..1_000 {
        let key = generate_personal_vault_key().expect("OS CSPRNG is available");
        let encoded = encode_personal_vault_key_for_storage(&key);
        assert!(encodings.insert(encoded[9..25].to_vec()));
    }

    let key = generate_personal_vault_key().expect("OS CSPRNG is available");
    let stored = encode_personal_vault_key_for_storage(&key);
    let restored = decode_personal_vault_key_from_storage(&stored).expect("valid stored key");
    let plaintext = b"\0binary\xff data with Unicode: \xf0\x9f\x8f\x8b";
    let encrypted = encrypt_blob(&key, plaintext, &binding()).expect("encryption succeeds");
    let encoded = encode_encrypted_blob(&encrypted);
    let decoded = decode_encrypted_blob(&encoded).expect("encoded envelope parses");

    assert_eq!(
        decrypt_blob(&restored, &decoded, &binding())
            .expect("decryption succeeds")
            .as_slice(),
        plaintext
    );
}

#[test]
fn encryption_uses_fresh_blob_keys_and_nonces() {
    let key = generate_personal_vault_key().expect("OS CSPRNG is available");
    let mut envelopes = BTreeSet::new();
    for _ in 0..1_000 {
        let encrypted =
            encrypt_blob(&key, b"same plaintext", &binding()).expect("encryption succeeds");
        assert!(envelopes.insert(encode_encrypted_blob(&encrypted)));
    }
}

#[test]
fn encryption_accepts_empty_and_sixteen_mebibyte_plaintexts_only() {
    let key = generate_personal_vault_key().expect("OS CSPRNG is available");
    let empty = encrypt_blob(&key, b"", &binding()).expect("empty plaintext encrypts");
    assert!(
        decrypt_blob(&key, &empty, &binding())
            .expect("empty plaintext decrypts")
            .is_empty()
    );

    let maximum = vec![0x5A; 16 * 1024 * 1024];
    let encrypted = encrypt_blob(&key, &maximum, &binding()).expect("maximum plaintext encrypts");
    assert_eq!(
        decrypt_blob(&key, &encrypted, &binding())
            .expect("maximum plaintext decrypts")
            .as_slice(),
        maximum
    );
    assert_eq!(
        error(encrypt_blob(&key, &[0; 16 * 1024 * 1024 + 1], &binding())),
        EncryptionError::InputTooLarge
    );
}

#[test]
fn fixed_vector_decrypts_and_preserves_its_canonical_encoding() {
    let key = decode_personal_vault_key_from_storage(&decode_hex(VECTOR_KEY))
        .expect("vector key is valid");
    let encoded = decode_hex(VECTOR_ENVELOPE);
    let blob = decode_encrypted_blob(&encoded).expect("vector envelope is valid");

    assert_eq!(encode_encrypted_blob(&blob), encoded);
    assert_eq!(
        decrypt_blob(&key, &blob, &binding())
            .expect("vector decrypts")
            .as_slice(),
        b"vector plaintext"
    );
}

#[test]
fn decoder_preserves_unknown_wrappers_byte_for_byte_and_in_order() {
    let key = decode_personal_vault_key_from_storage(&decode_hex(VECTOR_KEY))
        .expect("vector key is valid");
    let original = decode_hex(VECTOR_ENVELOPE);
    let mut encoded = original.clone();
    let payload_length_offset = 8 + 1 + 24;
    let payload_length = u32::from_be_bytes(
        encoded[payload_length_offset..payload_length_offset + 4]
            .try_into()
            .expect("four-byte payload length"),
    ) as usize;
    let wrapper_count_offset = payload_length_offset + 4 + payload_length;
    encoded[wrapper_count_offset] = 2;
    encoded.extend_from_slice(&decode_hex(VECTOR_UNKNOWN_WRAPPER));

    let decoded =
        decode_encrypted_blob(&encoded).expect("unknown wrapper remains structurally valid");
    assert_eq!(encode_encrypted_blob(&decoded), encoded);
    assert_eq!(
        decrypt_blob(&key, &decoded, &binding())
            .expect("known wrapper still decrypts")
            .as_slice(),
        b"vector plaintext"
    );
    assert_ne!(original, encoded);
}

#[test]
fn tampering_wrong_keys_and_wrong_bindings_fail_authentication() {
    let key = decode_personal_vault_key_from_storage(&decode_hex(VECTOR_KEY))
        .expect("vector key is valid");
    let other_key = generate_personal_vault_key().expect("OS CSPRNG is available");
    let encoded = decode_hex(VECTOR_ENVELOPE);
    let encrypted = decode_encrypted_blob(&encoded).expect("vector envelope is valid");
    let payload_length_offset = 8 + 1 + 24;
    let payload_length = u32::from_be_bytes(
        encoded[payload_length_offset..payload_length_offset + 4]
            .try_into()
            .expect("four-byte payload length"),
    ) as usize;
    let payload_offset = payload_length_offset + 4;
    let wrapper_offset = payload_offset + payload_length + 1;

    for offset in [
        9,
        payload_offset,
        payload_offset + payload_length - 1,
        wrapper_offset + 2,
        wrapper_offset + 18,
    ] {
        let mut tampered = encoded.clone();
        tampered[offset] ^= 1;
        let blob = decode_encrypted_blob(&tampered).expect("tampering leaves structure valid");
        assert_eq!(
            error(decrypt_blob(&key, &blob, &binding())),
            EncryptionError::AuthenticationFailed
        );
    }

    assert_eq!(
        error(decrypt_blob(&other_key, &encrypted, &binding())),
        EncryptionError::AuthenticationFailed
    );
    let mut altered_binding = binding();
    altered_binding.data_space_id[0] ^= 1;
    assert_eq!(
        error(decrypt_blob(&key, &encrypted, &altered_binding)),
        EncryptionError::AuthenticationFailed
    );
    altered_binding = binding();
    altered_binding.resource_type_id[0] ^= 1;
    assert_eq!(
        error(decrypt_blob(&key, &encrypted, &altered_binding)),
        EncryptionError::AuthenticationFailed
    );
    altered_binding = binding();
    altered_binding.resource_id[0] ^= 1;
    assert_eq!(
        error(decrypt_blob(&key, &encrypted, &altered_binding)),
        EncryptionError::AuthenticationFailed
    );
    altered_binding = binding();
    altered_binding.revision += 1;
    assert_eq!(
        error(decrypt_blob(&key, &encrypted, &altered_binding)),
        EncryptionError::AuthenticationFailed
    );
}

#[test]
fn truncation_and_declared_length_corpus_are_rejected_without_panicking() {
    let vector = decode_hex(VECTOR_ENVELOPE);
    for length in 0..vector.len() {
        assert!(decode_encrypted_blob(&vector[..length]).is_err());
    }

    let payload_length_offset = 8 + 1 + 24;
    for declared_length in [0_u32, 15, 16, 16 * 1024 * 1024 + 17] {
        let mut malformed = vector.clone();
        malformed[payload_length_offset..payload_length_offset + 4]
            .copy_from_slice(&declared_length.to_be_bytes());
        assert!(decode_encrypted_blob(&malformed).is_err());
    }

    let mut malformed_wrapper = vector;
    let wrapper_length_offset = 8 + 1 + 24 + 4 + 32 + 1 + 2 + 16 + 24;
    malformed_wrapper[wrapper_length_offset..wrapper_length_offset + 2]
        .copy_from_slice(&u16::MAX.to_be_bytes());
    assert!(decode_encrypted_blob(&malformed_wrapper).is_err());
}

#[test]
fn malformed_and_unsupported_envelopes_are_rejected_before_decryption() {
    assert_eq!(
        error(decode_encrypted_blob(b"GRDENC\0\x02")),
        EncryptionError::UnsupportedEnvelopeVersion
    );
    assert_eq!(
        error(decode_encrypted_blob(b"GRDENC\0\x01\x02")),
        EncryptionError::UnsupportedCipherSuite
    );
    assert_eq!(
        error(decode_encrypted_blob(b"bad")),
        EncryptionError::MalformedEnvelope
    );
    assert_eq!(
        error(decode_encrypted_blob(&vec![
            0;
            16 * 1024 * 1024 + 4 * 1024 + 1
        ])),
        EncryptionError::InputTooLarge
    );
}
