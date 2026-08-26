# `gradus-encryption`

Versioned, in-memory envelope encryption for Gradus vault blobs. It uses a random
personal vault key, a fresh random 32-byte blob key per blob, HKDF-SHA-256 key
derivation, and XChaCha20-Poly1305 authenticated encryption.

## Security boundary

`PersonalVaultKey` is secret and deliberately does not implement `Debug`,
`Display`, `Clone`, `Copy`, or serialization traits. The value returned by
`encode_personal_vault_key_for_storage` is also highly sensitive: persist it
only in OS-backed secure storage. Never upload, log, telemetry-report, or use
it as an ordinary backup.

This module does not access secure storage, files, keychains, or the network.
It zeroizes owned secret key material where Rust permits, but neither
zeroization nor secure storage can guarantee physical erasure in every runtime
or hardware path.

## Quick start

```rust
use gradus_encryption::{
    BlobBinding, decode_encrypted_blob, decrypt_blob, encode_encrypted_blob,
    encrypt_blob, generate_personal_vault_key,
};

fn encrypt_then_decrypt() -> Result<(), gradus_encryption::EncryptionError> {
    let key = generate_personal_vault_key()?;
    let binding = BlobBinding {
        data_space_id: [1; 16],
        resource_type_id: [2; 16],
        resource_id: [3; 16],
        revision: 1,
    };

    let encrypted = encrypt_blob(&key, b"workout bytes", &binding)?;
    let portable_bytes = encode_encrypted_blob(&encrypted);
    let downloaded = decode_encrypted_blob(&portable_bytes)?;
    let plaintext = decrypt_blob(&key, &downloaded, &binding)?;
    assert_eq!(plaintext.as_slice(), b"workout bytes");
    Ok(())
}
```

Generate and encode a key once, then pass the resulting sensitive bytes to the
platform's secure-storage adapter. On a later launch, retrieve the exact bytes
and call `decode_personal_vault_key_from_storage`.

## Protocol

- Plaintexts are in-memory only and limited to 16 MiB.
- All production randomness comes from the OS CSPRNG. A failure is returned as
  `EncryptionError::RandomnessUnavailable`; there is no fallback.
- `BlobBinding` is authenticated as
  `data_space_id[16] || resource_type_id[16] || resource_id[16] || revision_u64_be`.
  Use stable opaque 16-byte IDs and reconstruct the identical binding before
  decrypting.
- The payload AAD is `"gradus/encryption/v1/payload\0" || BlobBinding`.
  A symmetric-vault wrapper AAD is
  `"gradus/encryption/v1/key-wrap\0" || type || version || key_id || BlobBinding`.
- The wrapping key is HKDF-SHA-256 with the personal-vault secret as IKM, an
  empty salt, and info `gradus/encryption/v1/symmetric-blob-key-wrap`.

The sensitive key storage encoding is exactly:

```text
"GRDVKEY\0" || version_u8(1) || key_id[16] || secret[32]
```

The binary envelope is exactly:

```text
magic[8] = "GRDENC\0\1"
payload_suite_u8 = 1
payload_nonce[24]
payload_ciphertext_length_u32_be
payload_ciphertext
wrapper_count_u8
wrappers

wrapper = type_u8 || version_u8 || wrapping_key_id[16] || nonce[24]
          || wrapped_key_length_u16_be || wrapped_key_ciphertext
```

V1 emits one type-1/version-1 wrapper with a 48-byte wrapped blob-key
ciphertext. Decoding accepts 1–16 wrappers and retains structurally valid
unknown wrapper bytes in their original order, so
`encode_encrypted_blob(decode_encrypted_blob(bytes)?) == bytes`. Unknown
wrappers cannot decrypt in V1. The envelope is capped at `16 MiB + 4 KiB`.

Use unpadded base64url only when a text transport boundary requires it; this
crate intentionally exposes canonical bytes rather than a text transport.

## Errors and limitations

`AuthenticationFailed` deliberately covers wrong keys, wrong bindings, and
authentication failures without exposing cryptographic-library details. The
module rejects malformed or unsupported envelopes before decryption and never
returns partial plaintext.

AEAD detects modification and ciphertext substitution, but it does not prevent
a storage server from withholding, replaying, or observing ciphertext sizes and
timing. This V1 module has no password unlock, key recovery, multi-device
transfer, sharing, server authorization, streaming encryption, or secure
persistence.

