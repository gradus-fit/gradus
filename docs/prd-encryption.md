# PRD: Gradus encryption Core module

## Status

Proposed — research and design only. This PRD creates one future Core module, `core/encryption`; it does not change `core/openweight` or the server/client modules.

## Problem

Gradus must let a user create a workout locally, encrypt it before upload, and decrypt it only on their device. The server must store and synchronize opaque bytes, not workout data or plaintext decryption keys.

The first proof of concept is single-device. Its encrypted-blob format must nevertheless avoid a data migration when Gradus adds devices, recipient sharing, account delegation, recovery, or key rotation.

## Goal

Provide a small, platform-neutral Rust Core module that:

1. generates one random personal vault key per user vault, initially held by that user's sole authorized device;
2. encrypts and decrypts arbitrary in-memory blobs locally; and
3. produces a portable, versioned encrypted envelope that the server can store without interpreting.

**E2EE definition:** no plaintext workout, plaintext blob key, or plaintext personal vault key leaves a client. The future server may store *encrypted key wrappers* and public keys; it must never receive a raw decryption key.

## V1 scope and non-goals

### In scope

- Random personal vault-key generation.
- Envelope encryption/decryption for a workout blob.
- Authenticated binding of a ciphertext to a module-defined data-space/resource/revision identity.
- A documented binary envelope format and test vectors for all Core bindings.
- A minimal upload/download POC contract: client encrypts; server stores and returns bytes unchanged; client decrypts.

### Explicitly out of scope

- Password-derived unlocking, login/authentication, account registration, and user identity.
- Secure device persistence; each platform binding/application must use its OS secure-storage facility.
- Multi-device enrollment, recovery, public links, recipient sharing, roles, signatures, server authorization, sync conflict resolution, search, or deduplication.
- Streaming/file encryption. V1 accepts at most 16 MiB of in-memory plaintext.
- Server-side validation of workout content. `docs/data_model.md` is currently TBD, so this PRD does not assume a workout serialization.

## Decision: cryptographic construction

Use a simple, versioned envelope-key hierarchy modeled on Ente's collection/file keys and Atuin's per-record envelope keys:

```text
PersonalVaultKey (random 16-byte key ID + random 32-byte secret; one per user vault)
  └─ HKDF-SHA-256
       └─ wrapping key
            └─ encrypts a fresh random 32-byte BlobKey per blob
                 └─ XChaCha20-Poly1305 encrypts the workout bytes
```

- **Randomness:** Production public APIs use the OS CSPRNG only. Failure to obtain randomness is an error; there is no fallback. The vault key ID is public routing metadata; its secret bytes are not.
- **Test-vector randomness:** A private test-only encryption path MAY accept a deterministic source for fixed key/nonce test vectors. It must not be public, callable by a production binding, or enabled through a production feature flag. The production encryption path always obtains randomness from the OS CSPRNG.
- **Payload and key wrapping:** XChaCha20-Poly1305 AEAD, with a new random 24-byte nonce for every encryption. Ciphertext includes the 16-byte authentication tag.
- **Key derivation:** derive the 32-byte symmetric wrapper key with `HKDF-SHA-256(ikm = personal-vault-secret, salt = empty, info = ASCII "gradus/encryption/v1/symmetric-blob-key-wrap")`. HKDF is only domain separation, never a password KDF.
- **Envelope encryption:** each blob has a fresh `BlobKey`. The personal vault key wraps that BlobKey instead of directly encrypting content. The envelope contains an extensible, length-delimited wrapper list (V1 encryption emits exactly one symmetric-vault wrapper). A later client may add a recipient/link/shared-domain wrapper while preserving the encrypted payload.
- **Associated data (AAD):** `BlobBinding` is exactly `data_space_id[16] || resource_type_id[16] || resource_id[16] || revision_u64_be`. `data_space_id` identifies a personal or shared encrypted data domain, not a server account; `resource_type_id` is an opaque, stable type ID; and `resource_id` identifies that logical resource. Payload AAD is `ASCII "gradus/encryption/v1/payload\0" || BlobBinding`. A wrapper AAD is `ASCII "gradus/encryption/v1/key-wrap\0" || wrapper_type_u8 || wrapper_version_u8 || wrapping_key_id[16] || BlobBinding`. Thus immutable payload identity is distinct from mutable wrapper/key identity; an update with a new revision is a new blob. Rich metadata belongs encrypted in the payload, not in AAD.
- **Encoding:** the canonical envelope is `magic[8] = "GRDENC\0\1" || payload_suite_u8 = 1 || payload_nonce[24] || payload_ciphertext_length_u32_be || payload_ciphertext || wrapper_count_u8 || wrappers`. A wrapper is `type_u8 || version_u8 || wrapping_key_id[16] || nonce[24] || wrapped_key_length_u16_be || wrapped_key_ciphertext`. V1 accepts 1–16 wrappers; it emits one type-1/version-1 wrapper whose wrapped key ciphertext is exactly 48 bytes. Reject an envelope when its total byte length exceeds 16 MiB + 4 KiB, a declared length exceeds bytes remaining/the stated bound, or a known field has an invalid length. Unknown length-delimited wrapper types cannot decrypt in V1, but Core preserves each wrapper's original encoded bytes and list order. Therefore, for every structurally valid envelope, `encode_encrypted_blob(decode_encrypted_blob(bytes))` must equal `bytes` byte-for-byte, including unknown wrappers. Future re-wrapping may append or replace recognized wrappers only; it must retain unknown wrappers in place. Binary-to-text transport uses unpadded base64url only at an API boundary. Do not use ad-hoc JSON serialization as cryptographic input.

XChaCha20-Poly1305 is an AEAD, so it detects wrong keys, tampering, and binding/ciphertext substitution. It does **not** prevent a malicious server from withholding data, replaying a previously valid blob, observing ciphertext length/timing, or copying data a legitimate recipient already decrypted.

## V1 public interface

All public Rust items belong in `src/api.rs` and are re-exported by `src/lib.rs`. Secret types must not implement `Debug`, `Display`, `Serialize`, `Deserialize`, `Clone`, or `Copy`.

```rust
pub struct PersonalVaultKey { /* 16-byte ID + 32 secret bytes */ }
pub struct BlobBinding {
    pub data_space_id: [u8; 16],
    pub resource_type_id: [u8; 16],
    pub resource_id: [u8; 16],
    pub revision: u64,
}
pub struct EncryptedBlob { /* versioned portable envelope */ }

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

pub fn generate_personal_vault_key() -> Result<PersonalVaultKey, EncryptionError>;
pub fn decode_personal_vault_key_from_storage(
    encoded: &[u8],
) -> Result<PersonalVaultKey, EncryptionError>;
pub fn encode_personal_vault_key_for_storage(
    key: &PersonalVaultKey,
) -> Zeroizing<Vec<u8>>;
pub fn decode_encrypted_blob(encoded: &[u8]) -> Result<EncryptedBlob, EncryptionError>;
pub fn encode_encrypted_blob(blob: &EncryptedBlob) -> Vec<u8>;

pub fn encrypt_blob(
    key: &PersonalVaultKey,
    plaintext: &[u8],
    binding: &BlobBinding,
) -> Result<EncryptedBlob, EncryptionError>;

pub fn decrypt_blob(
    key: &PersonalVaultKey,
    encrypted: &EncryptedBlob,
    binding: &BlobBinding,
) -> Result<Zeroizing<Vec<u8>>, EncryptionError>;
```

`EncryptionError` is a stable public error contract, marked `#[non_exhaustive]` so future formats can add variants. Its `Display` output must contain no plaintext, key bytes, AAD/binding values, envelope bytes, or underlying cryptographic-library detail. `AuthenticationFailed` is deliberately one outcome for wrong vault keys, wrong bindings, altered nonces/ciphertext/tags/wrappers, and corrupted ciphertext; callers must not be able to distinguish those cases.

Secure persistence is exclusively the client/platform binding's responsibility. Core supplies `encode_personal_vault_key_for_storage` and `decode_personal_vault_key_from_storage` only as pure encoding/validation functions, so every binding uses the same compatible key representation across app restarts. The client passes the encoded bytes to its OS secure-storage facility and later supplies the retrieved bytes to Core; Core never accesses a keychain, writes a file, logs key material, or chooses storage.

The canonical, sensitive encoding is exactly `"GRDVKEY\0" || version_u8(1) || key_id[16] || secret[32]` (57 bytes); decode accepts only that length/version. The encoded bytes must be explicitly documented as highly sensitive and must never be uploaded, logged, or used as an ordinary backup.

The module validates the envelope version, field lengths, encoding, and authentication before returning plaintext. It rejects unknown envelope versions and never silently downgrades algorithms or formats. `decode_encrypted_blob` applies the stated envelope limit before allocating payload or wrapper storage.

## POC workflow and server contract

1. The app creates a `PersonalVaultKey` once and saves it in secure local storage. It is never uploaded.
2. The app allocates 16-byte opaque data-space, resource-type, and resource IDs plus an unsigned 64-bit revision before encryption. A personal vault is one data space; a teacher/student shared vault is another.
3. It constructs `BlobBinding`; the module's fixed encoding prevents ambiguous cross-platform concatenation, type confusion, and server-side ciphertext movement between data spaces or records.
4. It serializes `EncryptedBlob` with `encode_encrypted_blob` and uploads those opaque bytes, IDs/revision, and unavoidable account/auth metadata.
5. On download, the app decodes with `decode_encrypted_blob`, reconstructs the same `BlobBinding`, and decrypts locally. The server cannot parse or validate the workout.

The POC server endpoint must treat the envelope as opaque and preserve bytes exactly. TLS remains mandatory but is transport protection, not a substitute for E2EE.

This PRD completes only the cryptographic Core. A runnable E2EE POC additionally needs **separate, sequenced PRDs** for (1) a Core platform-binding module and secure-storage adapter, (2) an opaque encrypted-blob endpoint in the Server vault module, and (3) a Client workout feature that constructs `BlobBinding`. Gradus's one-module-at-a-time rule means these are not one implementation change. "Core complete" is not "POC complete" until those follow-ups have been implemented and tested end-to-end.

## Future-compatible extensions (not V1 work)

The V1 per-blob key envelope is the intentional seam for these features:

| Need | Later design direction | Consequence to retain now |
|---|---|---|
| Multiple devices | A personal vault key is copied only to authorized devices. A new device creates device X25519 and Ed25519 keys; an unlocked device transfers/wraps the existing vault key through an authenticated, expiring pairing session and shows a human-verifiable fingerprint/QR. | Do not bake a device identifier into the blob key or payload ciphertext. |
| Password unlock | Derive a wrapping key with Argon2id and a unique 16+ byte salt; store parameters with the password-wrapped personal vault key. Password changes re-wrap the vault key, not every blob. RFC 9106's memory-constrained recommendation is Argon2id with 64 MiB and 3 passes; calibrate per supported platform and never silently weaken stored parameters. | The personal vault key is random and independent of a password. |
| One-time recovery | Generate a high-entropy recovery secret/mnemonic client-side and use it to wrap the personal vault key. A server-enforced one-time redemption record can control workflow, but a server-known OTP alone cannot decrypt E2EE data. | Do not claim a normal email/SMS OTP is cryptographic recovery. |
| Share an individual workout/collection | Generate a recipient X25519 public-key envelope or a random link key that wraps the existing BlobKey/collection key. Public link form: `https://…/share/<opaque-id>#k=<base64url-link-key>`; the fragment is not sent in HTTP requests. | Add a wrapper; do not alter payload ciphertext. |
| Share an account with a teacher | Create a separately named shared data domain/key and wrap it to each approved device/recipient. Server ACLs enforce API writes/reads; cryptography grants decryption. Roles alone cannot revoke data already copied by a recipient. | Personal-key envelopes and future shared-domain envelopes need the same BlobKey wrapper abstraction. |
| Authenticity and rollback detection | Give each device an Ed25519 signing key. Sign canonical encrypted operations and bind immutable record metadata as AAD; use versioned append-only operations/tombstones and conflict rules. Verify public-key changes through pairing/fingerprints or a key-transparency design. | AEAD prevents tampering, but a valid older ciphertext is still valid. |
| Large files | Add a distinct, versioned streaming envelope using `crypto_secretstream_xchacha20poly1305` semantics: authenticated chunks, final marker, and authenticated metadata. | Never reinterpret V1 in-memory envelopes as streams. |

Public-link viewers must disable/avoid third-party resources and use a strict referrer policy: a fragment normally stays client-side, but application code can still leak it through analytics, copied URLs, or script behavior.

## Security and implementation requirements

- Use maintained, audited implementations of the chosen primitives; do not implement cryptographic algorithms.
- Verify Rust, Python, Dart, and Wasm interoperability with the same deterministic test vectors before declaring a binding supported. Library selection is an implementation spike, not an excuse to change this wire protocol.
- Use `zeroize`/`Zeroizing` for owned secret material where Rust permits it. Document that zeroization and OS secure storage reduce exposure but cannot guarantee erasure in every runtime or hardware path.
- Never include plaintext, key bytes, recovery material, or decryptable metadata in logs, errors, telemetry, crash reports, filenames, or visible server fields.
- Do not expose a plaintext digest to the server: it leaks equality of workouts. AEAD already provides ciphertext integrity; any future semantic digest must be encrypted or keyed and separately specified.
- Re-wrapping a BlobKey is operational key rotation, not retrospective compromise recovery: a server retaining an old wrapper and ciphertext can still be read by a leaked old vault key. Protecting existing records after key compromise requires fresh blob keys and payload re-encryption; no design can revoke plaintext or ciphertext already copied by an adversary.
- The server-visible envelope necessarily leaks ciphertext size, update timing, account relationship, and access patterns. Mitigations such as padding, batching, private discovery, and traffic obfuscation are future product decisions.
- Threat-model a malicious storage server, database breach, passive network attacker, device theft, and ciphertext tampering. A compromised unlocked client or a recipient who copied plaintext is out of scope for E2EE alone.

## Acceptance criteria and test plan

- Generate 1,000 keys and blob keys without duplication in a statistical smoke test; production correctness relies on the CSPRNG, not that test.
- Round-trip empty, binary, Unicode, and exactly 16 MiB blobs; reject a 16 MiB + 1 byte plaintext and every envelope above the specified 16 MiB + 4 KiB bound.
- Generate/export/import/decrypt with the fixed key encoding, and serialize/parse every envelope field with fixed cross-binding vectors.
- Verify `BlobBinding` preserves arbitrary 16-byte IDs byte-for-byte in AAD vectors.
- The same plaintext encrypted twice yields different payload and wrapped-key ciphertexts.
- Decryption rejects any change to version, nonce, ciphertext, tag, wrapped blob key, wrapper key ID, or binding; it returns no partial plaintext.
- A ciphertext cannot decrypt under another personal vault key or under a binding with a different data space, resource type, resource ID, or revision.
- Malformed/truncated/oversized envelopes never panic or allocate unbounded memory.
- Publish fixed test vectors for envelope serialization, encryption, and each tampering case; consume them from Rust plus every supported binding. Include envelopes with unknown wrappers and assert byte-for-byte decode/encode round trips with wrapper ordering retained.
- Fuzz envelope parsing and decryption inputs; test errors contain no secret/plaintext material.
- Before implementation, copy `core/template`, include README and public-root integration tests, update `docs/core.mmd`, run public-API coverage, and run repository `pre-commit`.

## Research learnings and sources

- **Ente** is the closest structural match: a random master key encrypts collection keys, which encrypt file keys; it uses Argon2id for password-derived wrapping, libsodium secretbox/secretstream for data, X25519 sealed boxes for sharing, recovery-key wrapping, and URL fragments for public links. Its key hierarchy is why this PRD adopts a random per-blob key now. Official architecture: <https://ente.com/architecture/>. Inspected repository commit `a0374add988a`: `web/packages/accounts/services/user.ts`, `web/packages/base/crypto/libsodium.ts`, and `docs/docs/locker/features/sharing/public-links.md`.
- **Bitwarden** demonstrates separation of a random user key from the master password, per-item cipher keys, public-key key exchange for organizations/emergency access, and Send link seeds kept in URL fragments. It also shows the operational cost of wrapping a root/user key for recovery and rotation. It uses older AES-CBC+HMAC/RSA constructions, so this PRD does not copy its wire format. Official whitepaper: <https://bitwarden.com/help/bitwarden-security-white-paper/>. Inspected commits `571f8806fcf7` (server) and `093b81451f10` (clients).
- **Atuin** uses an envelope-encrypted random content-encryption key per record and authenticates record metadata as PASETO implicit assertions. This validates treating metadata binding as first-class rather than trusting server-side record placement. It intentionally has no server-mediated key recovery/sharing; a lost key is lost. Inspected commit `1129b036d8bc`: `crates/atuin-common/src/encryption/paseto_v4.rs` and `crates/atuin-domain/src/record/mod.rs`.
- **Standards:** RFC 9106 (Argon2) <https://www.rfc-editor.org/rfc/rfc9106>; libsodium secretstream documentation <https://libsodium.gitbook.io/doc/secret-key_cryptography/secretstream>. These guide future password and streaming work, not the V1 no-password POC.
