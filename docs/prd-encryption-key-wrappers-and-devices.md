# PRD: Gradus encryption key wrappers, recovery, and account keys

## Status

Protocol design approved; implementation blocked pending the required cross-platform Argon2id benchmark. The companion [key-wrapper protocol V1](protocol-encryption-key-wrappers-v1.md) defines record layouts, AAD, encodings, and validation rules. It extends the existing `core/encryption` module after its personal-vault/blob envelope API and adds no Server, Client, binding, authentication, or persistence implementation.

**Compatibility decision:** `core/encryption` currently has no consumers outside its own README example, integration tests, and fuzz targets. No Client, Server, binding, or other Core module imports it, and no encrypted records have been deployed. This round may therefore make breaking changes to its public API, local-key encodings, and encrypted-envelope formats. The implementation must update the module README, tests, fuzz targets, and vectors together; backward decoding is not required until a consumer/deployed format exists.

This revision incorporates decisions from the Ente implementation review at commit `813fc66` (2026-03-31).

## Problem

The current encryption module generates a random `PersonalVaultKey` (PVK) and requires its sensitive `GRDVKEY` encoding to remain in local OS secure storage. That is correct for a single-device POC, but it cannot let a user unlock the same encrypted vault in the web app and on multiple phones.

A password cannot be the PVK: it has lower entropy, may change, and must not require re-encrypting every existing blob. The server needs to synchronize encrypted root-key material and public keys without learning a usable vault key.

## Goals

Extend `core/encryption` with portable, versioned cryptographic records that let a client:

1. wrap its random PVK with a key derived locally from the user's password;
2. unwrap that same PVK on another client after normal account authentication authorizes downloading the wrapper;
3. generate a high-entropy recovery key and recover the same PVK without the password;
4. generate an account-level X25519 key pair, encrypt its private key with the PVK, and make its public key available for future sharing and encrypted server-token delivery; and
5. change a password by atomically replacing the current password wrapper, while retaining the PVK, recovery key, account key pair, and all blob ciphertexts.

The server may store opaque wrapper bytes, the account X25519 public key, wrapper routing IDs, and normal account/session authorization metadata. It must never receive a plaintext password, password-derived key-encryption key (KEK), PVK, recovery key, account private key, or decrypted blob key.

## Non-goals

This PRD does **not** define or implement:

- account registration, login, sessions, passkeys, MFA, email/phone verification, password-verifier storage, PAKE, or API authorization;
- server tables/endpoints, password-reset workflow, recovery UI, rate limits, audit controls, or device-management UX;
- persistent device cryptographic key pairs, device key wrappers, or device-to-device pairing;
- secure-storage adapters, Rust platform bindings, Dart/Python/Wasm bindings, or Client code;
- sharing a vault/blob key with another account; the account public key only prepares for that future work;
- signed data operations, conflict resolution, key transparency, or rollback detection;
- re-wrapping existing per-blob keys, changing `EncryptedBlob`, or changing its V1 binary format.

A wrapper proves only possession of the decryption secret. It never authenticates an HTTP request. Server authorization and E2EE decryption authorization remain separate concerns.

## Terminology and key hierarchy

- **PersonalVaultKey (PVK):** Existing random 16-byte key ID plus random 32-byte secret. It decrypts the symmetric wrappers in Gradus blob envelopes.
- **Password KEK:** Transient 32-byte Argon2id output derived locally from the password. It wraps the PVK and is neither uploaded nor reused for authentication, signing, or general encryption.
- **RecoveryKey:** Random 32-byte secret generated client-side. Its portable user representation is a 24-word BIP-39 mnemonic in a caller-selected official BIP-39 language, with no BIP-39 passphrase. It wraps the PVK for recovery and is itself encrypted with the PVK for later display by an already-unlocked client.
- **Account encryption key pair:** One X25519 public/private key pair for the encrypted account, not per device. Its public key is server-visible; its private key is encrypted under the PVK. It is the future recipient key for account-to-account sharing and may decrypt an API token sealed to the account public key.
- **Vault ID:** A random stable 16-byte, client-generated public ID for the personal encrypted data domain. It is the `data_space_id` used by personal-vault blob bindings.
- **Key wrapper:** A versioned authenticated encrypted record containing a secret. A server may store it but it remains security-sensitive: a password wrapper enables offline password guesses, and recovery/account-private-key wrappers are decryptable by a holder of their respective secret.

```text
PersonalVaultKey (random and independent of password)
 ├─ existing HKDF-derived symmetric wrapper → fresh BlobKey → blob payload
 ├─ Argon2id password KEK → password PVK wrapper
 ├─ RecoveryKey → recovery PVK wrapper
 └─ PVK-derived account-key wrapper → account X25519 private key

PVK → recovery-key wrapper, so an unlocked client can show the recovery key again.
```

Changing the password replaces only the password PVK wrapper. It must not generate a PVK, recovery key, or account key pair, nor re-encrypt blobs. A leaked PVK remains able to decrypt copied historical ciphertext; changing wrappers alone is not retrospective compromise recovery.

## Design decisions

### 1. Password unlock is local and distinct from server authentication

During account/vault creation, the client generates the PVK, derives a password KEK locally, creates a password PVK wrapper, and uploads only that wrapper after ordinary account registration/authentication permits it.

On another client, normal account authentication authorizes downloading the current wrapper. The client obtains the password, derives the KEK locally, and decrypts the PVK locally. Wrong password, modified wrapper, wrong vault ID, and authentication failure have one public result: `AuthenticationFailed`.

The future authentication PRD must use a separate construction and separate salts from password vault unlock. It must not reuse the password KEK as an API token, password verifier, signing key, or general encryption key. If password authentication is offered, that PRD should evaluate OPAQUE (RFC 9807) rather than inventing a PAKE. This intentionally differs from Ente's current SRP flow, which derives an SRP login subkey from its password KEK.

**Trade-off:** separate unlock/authentication constructions add implementation and UX complexity: users may need to satisfy an API-authentication step and then unlock the vault. They prevent accidentally turning the vault-unlock KEK into a general-purpose server authentication secret and permit future passkeys/MFA without changing encrypted data. The server cannot grant decryption through email verification, password reset, support action, passkey, or MFA alone.

### 2. Password KDF policy and password input

The password wrapper uses Argon2id and stores a randomly generated salt of at least 16 bytes, memory cost in KiB, iteration count, and parallelism. These values are public but authenticated as wrapper associated data.

The initial supported security target is at least **4 GiB of memory × iterations**, parallelism 1, with a minimum memory cost of **128 MiB**. The preferred profile is 1 GiB memory and 4 iterations. A client may select only a supported, benchmarked profile that meets both floors; its selected parameters are persisted in the wrapper. Before any supported platform ships, the Core/binding spike must benchmark the profile on the weakest supported web, Android, iOS, and desktop hardware and publish an allowlist of safe profiles.

Clients must reject a wrapper below the security floor, outside the allowlist, or above explicit local resource caps before allocating/KDF work. They must never silently lower a stored wrapper's parameters to make unlock succeed. Registration/password change fails clearly if the platform cannot satisfy a supported profile.

Because a stolen password wrapper permits offline guesses, the future Client/auth work must require a password-strength check at creation/change (at least a locally evaluated `zxcvbn` score of 3) and encourage a password-manager-generated password. Strength scoring is a UX guard, not a replacement for Argon2id.

**Platform decision:** password unlock must support Web/Wasm, Android, iOS, and desktop before Gradus declares it supported. A Core/binding benchmark on the weakest supported hardware must publish the finite KDF allowlist and explicit resource caps before implementation. Until then, no client may emit or accept a password wrapper; the protocol reserves the parameter fields but deliberately does not choose numerical profiles.

Password bytes are the exact UTF-8 encoding of the entered Unicode scalar-value sequence, with **no Unicode normalization, trimming, case conversion, or locale transformation**. Every binding must use that rule and fixed tests must cover ASCII plus composed/decomposed Unicode inputs. This avoids silent cross-platform changes: visually equivalent but differently encoded passwords are deliberately different passwords.

A password wrapper encrypts the exact existing 57-byte `GRDVKEY` encoding, not only the PVK secret. Decryption passes that plaintext through `decode_personal_vault_key_from_storage`, so PVK version and key ID receive uniform validation.

### 3. Password changes atomically replace current wrappers

A password change derives a new Argon2id KEK with a new salt and supported parameters, wraps the *existing* PVK, then atomically replaces the current password wrapper for that vault/key version. The server must not normally retain an active password wrapper decryptable by the old password.

The future server/auth PRD must make the password-authentication credential update, the password-wrapper replacement, and session/token revocation one transaction or a recoverable, idempotent protocol with no state that accepts a new password while serving the old active wrapper. The Server must reject stale wrapper generations. Tests must show that the old password cannot unlock the wrapper returned by normal server state after a successful change.

No cryptographic design can erase an old wrapper that a malicious server, backup, or attacker already copied. If password compromise may include a leaked PVK, changing the password is insufficient; full recovery requires PVK/data rotation in later work.

### 4. Recovery is a V0 requirement

At vault creation, the client generates a random 32-byte `RecoveryKey`, converts it to a 24-word mnemonic in a caller-selected official BIP-39 language, and requires the user to save it before the vault is considered configured. V1 supports the ten official BIP-39 wordlists and no BIP-39 passphrase. The selected language is presentation metadata, not server/wrapper data; V1 conversion requires an explicit language and does not auto-detect it. The mnemonic is displayed only in deliberate recovery-key UI and must never enter logs, telemetry, crash reporting, filenames, clipboard history by default, or server fields.

The client creates two independent authenticated wrappers:

- **recovery PVK wrapper:** the RecoveryKey encrypts the canonical PVK encoding, allowing a client that has passed the future account-recovery authorization flow to unlock the vault; and
- **PVK recovery-key wrapper:** the PVK encrypts the RecoveryKey, allowing an already-unlocked client to retrieve/display its recovery key later without generating a new one.

A recovery flow can restore the PVK and let the user create a new password wrapper. It may restore server-account access only according to the future auth policy. An email/SMS OTP or support intervention alone is not an E2EE recovery secret and cannot decrypt the PVK.

The recovery key has full vault-decryption authority. Replacing it requires creating a new recovery key and atomically replacing both recovery wrappers; it does not invalidate data already copied by someone with the old PVK/recovery key.

### 5. Account-level X25519 keys, not persistent device keys

At account/vault creation, the client generates one account-level X25519 key pair. The server stores the public key. The private key is encrypted locally with a PVK-derived wrapping key and stored as an opaque account-private-key wrapper; any client that has unlocked the PVK may retrieve and decrypt it.

This is intentionally not a persistent per-device key architecture. A user gains multi-device access by authenticating to the account and unlocking the same PVK with the password or recovery key. Future device registration/revocation remains an API/session authorization concern, not a cryptographic promise to revoke data already unlocked by a removed device.

The account key pair prepares Gradus for a future sharing protocol: a sender can encrypt a resource/collection key to the recipient account's public key, and any recipient client unlocked with the recipient PVK can decrypt the account private key. It can also support Ente-like delivery of a newly-issued API token sealed to the account public key. Neither use is implemented by this PRD.

Ed25519 signing keys are not included. A later authenticity/rollback design must decide whether signing identities are account-level, device-level, or both, before generating durable signing keys.

### 6. Bind every wrapper to the client-generated vault identity

Every wrapper authenticates its complete cleartext header, wrapper type/version, `vault_id`, and PVK key ID as associated data. Account-private-key wrappers additionally bind the X25519 public key. Unwrap APIs must receive the expected vault/key identities from the caller; on a client that has securely stored those identities, this rejects cross-vault or cross-key substitution.

A fresh client cannot obtain whole-state substitution protection by trusting a `vault_id`, PVK key ID, account public key, and wrapper supplied together by an untrusted server: a self-consistent state authenticates against itself. Initial account authentication plus password/recovery knowledge makes server fabrication difficult, but does not detect all equivocation or replay. A future verified account-key fingerprint, signatures/key transparency, and append-only synchronization address that stronger threat.

Do not bind wrappers to a mutable email address, display name, device name, or server database primary key. The future auth/account module may define an immutable account subject for server authorization, but it is not cryptographic ownership of a vault.

### 7. Public-key sharing encryption is deliberately deferred

This PRD freezes the account X25519 key purpose, but not the future public-key wrapper algorithm or wire encoding. A sharing PRD must run a mandatory Rust/Dart/Python/Wasm interoperability and maintenance spike before selecting a suite.

| Option | Benefits | Costs / limits |
|---|---|---|
| RFC 9180 HPKE (`DHKEM(X25519, HKDF-SHA256)`, HKDF-SHA256, ChaCha20-Poly1305) | IETF-standard ciphersuite identifiers; explicit `info` and AAD binding; defined mode/suite agility; a clean path to future hybrid suites; maintained Rust and Python implementations exist. | The base mode does not authenticate the sender; Flutter/Dart implementation maturity and cross-binding vectors must be proven; uses 12-byte ChaCha20-Poly1305 nonces rather than this module's XChaCha payload suite. |
| libsodium `crypto_box_seal` / sealed boxes | Very simple, mature high-level API; broad libsodium binding availability; Ente deploys it for recipient key sharing. | Fixed X25519/XSalsa20-Poly1305 construction; intentionally anonymous (no sender authentication); no external AAD API, so metadata binding needs an additional authenticated inner record; less explicit algorithm agility. |

Neither primitive by itself proves sender identity or solves malicious-directory key substitution. The sharing design must provide verified recipient key changes and signatures/authorization where required. Post-quantum/hybrid encryption is not a V0 priority, but all wrapper formats need a version and suite identifier so a later migration can add a hybrid suite without reinterpreting old bytes.

## Public API extension

All public types/functions belong in `core/encryption/src/api.rs` and are re-exported by `src/lib.rs`; implementations stay under `src/private/`. Per the compatibility decision above, this implementation round may replace the existing public API and `EncryptedBlob` encoding rather than preserving compatibility.

The final type/function names and exact encoded fields are deliberately deferred pending the Core/binding implementation spike. The eventual public interface must provide the following capabilities:

```rust
pub struct VaultId(/* 16-byte public identifier */);
pub struct PasswordKdfParameters { /* Argon2id parameters */ }
pub struct PasswordVaultKeyWrapper { /* portable versioned record */ }
pub struct RecoveryKey { /* random 32-byte secret */ }
pub struct RecoveryVaultKeyWrapper { /* portable versioned record */ }
pub struct RecoveryKeyWrapper { /* PVK-encrypted recovery key */ }
pub struct AccountPublicKey { /* X25519 public key and key ID */ }
pub struct AccountKeyPair { /* X25519 private/public key pair */ }
pub struct AccountPrivateKeyWrapper { /* PVK-encrypted private key */ }

// Generate/encode/decode VaultId, RecoveryKey, and account key pair.
// Convert RecoveryKey to/from a caller-selected official BIP-39 24-word mnemonic.
// Password-wrap/unwrap the PVK.
// Recovery-wrap/unwrap the PVK, and PVK-wrap/unwrap the RecoveryKey.
// PVK-wrap/unwrap the account X25519 private key.
// Canonically encode/decode every server-stored wrapper.
// Canonically encode/decode private keys only for local secure storage.
```

`PersonalVaultKey`, `RecoveryKey`, `AccountKeyPair`, passwords, derived KEKs, PVK plaintext encodings, and account private-key storage encodings must not implement `Debug`, `Display`, `Clone`, `Copy`, `Serialize`, or `Deserialize`. `VaultId`, public keys, KDF parameters, and encrypted wrapper types may implement safe comparison/serialization traits.

`EncryptionError` may gain only necessary non-sensitive variants, such as `UnsupportedKeyWrapperVersion`, `UnsupportedKeyWrapperSuite`, `UnsupportedKdfParameters`, `InvalidRecoveryKeyEncoding`, and `InvalidAccountKeyEncoding`. Wrong password/recovery key, wrong vault ID, wrong PVK/account key, altered authenticated header, malformed ciphertext, and AEAD failure must converge to `AuthenticationFailed` after structurally safe parsing.

## Wire-format decision and implementation gate

The companion [key-wrapper protocol V1](protocol-encryption-key-wrappers-v1.md) supplies the field layouts, magic values, size limits, AAD byte sequences, recovery mnemonic rules, and test-vector requirements. It uses separate binary formats for password PVK wrappers, recovery PVK wrappers, PVK recovery-key wrappers, and account-private-key wrappers. It does not overload the existing `EncryptedBlob` wrapper format, which wraps per-blob keys and has a different lifecycle.

No implementation may begin until the companion protocol's required cross-platform Argon2id benchmark ratifies its finite parameter allowlist and resource caps. The resulting specification must retain canonical byte-for-byte decode/encode behavior, strict bounded parsing before allocation/KDF work, version/suite identifiers, all authenticated header bytes, and cross-binding deterministic test vectors.

The existing `GRDVKEY` local PVK encoding remains forbidden from upload. Server-stored wrappers are not ordinary backup files.

## Server/client contract for later PRDs

A future Server module may store unchanged password/recovery/account-private-key wrapper bytes and account public-key metadata. It authorizes create/read/replace operations by account/session identity, never by an ability to decrypt. It must not decrypt, re-encrypt, or manufacture wrappers. It must enforce active-wrapper generations and atomic password/recovery update semantics after those records are specified.

A future Client module stores PVKs and account private keys only in OS secure storage, locally derives password KEKs, presents and confirms recovery mnemonic handling, and keeps cleartext keys out of logs/analytics/crash reporting. It must not claim that a normal password reset restores encrypted data without a recovery key or an already-unlocked client.

## Security requirements

- Use maintained audited Argon2id, X25519, XChaCha20-Poly1305, BIP-39, and secure-randomness implementations; never implement primitives manually.
- Generate PVKs, recovery keys, vault IDs, salts, nonces, and X25519 keys from the OS CSPRNG; fail closed when randomness is unavailable.
- Zeroize owned password bytes where bindings permit, plus derived KEKs, PVK/recovery-key plaintext encodings, and account private keys. Document language/runtime limitations.
- The wrapper parse/decrypt boundary accepts untrusted server bytes. Enforce fixed limits for every field and KDF parameter before allocation or Argon2id work.
- Do not expose plaintext secrets or distinguish wrong password/recovery key/tampering in errors, logs, analytics, telemetry, crashes, or server responses.
- A malicious server can observe account relationships, ciphertext/wrapper sizes, update timing, and access patterns, and can deny service/replay old valid state. Wrapper AEAD does not solve these problems.

## Acceptance criteria after the protocol specification exists

- Password and recovery wrapping round-trip random PVKs; restored PVKs decrypt pre-existing blob envelopes.
- Password changes preserve the PVK and blob access while making the old password fail against normal active server state.
- Recovery unwraps the PVK after password loss and permits a new password wrapper without data re-encryption.
- Same secret wrapped twice yields independently randomized records.
- Wrong password/recovery key/vault ID/key identity, altered header/KDF parameters/salt/nonce/ciphertext/tag, truncation, unknown suite, and unsupported version never return a secret and emit only the prescribed non-sensitive error.
- Decoders reject unsafe-low/out-of-policy KDF settings and oversized/malformed records before unbounded allocation or KDF work.
- Account key pairs round-trip through PVK wrapping; a client unlocking the same PVK on another platform obtains the same account private key and public-key identity.
- Fixed deterministic vectors cover password byte conversion, Argon2id, all wrapper encodings, every supported recovery-mnemonic wordlist, tampering, and malformed inputs. Rust, Dart, Python, and Wasm consume the vectors before support is declared.
- Fuzz every wrapper decoder/decryption path; test no panic, bounded allocation, and no secret/plaintext in errors.

## Sequenced follow-up PRDs

The one-module-at-a-time rule applies. This PRD is Core cryptographic design only. A runnable V0 needs separately sequenced work:

1. **Cross-platform Argon2id KDF benchmark spike:** research-only benchmark harnesses for the weakest supported Web/Wasm, Android, iOS, and desktop hardware; candidate profiles; timing, memory, allocation-failure, and battery/foreground measurements; published resource caps and a shared allowlist. This must be completed without production wrapper implementation.
2. **Core encryption key wrappers/recovery/account keys:** ratify the benchmark's profile table, then implement Core, bindings, vectors, secure-memory behavior, and `core/encryption` documentation in module-sequenced work.
3. **Server authentication and authorization:** immutable account subjects; account authentication/session strategy; passkeys/MFA; password-auth choice; rate limits; session revocation; and atomic credential/wrapper replacement.
4. **Server opaque vault-key storage:** authorized storage of current wrapper generations and account public keys.
5. **Client vault unlock/recovery:** secure storage, password unlock, password change, recovery-key presentation/confirmation/recovery, and lost-device UX.
6. **Sharing and account-key verification:** recipient directory, public-key change verification, sharing wrappers, and the HPKE-versus-sealed-box interoperability decision.
7. **Authenticity/rollback:** signatures, append-only operations, conflict rules, and potentially key transparency.

## Research and sources

- Existing Gradus design: `docs/prd-encryption.md` and `core/encryption`; the random PVK/per-blob-key hierarchy remains unchanged.
- Ente source review at commit `813fc66`: `web/packages/accounts/services/user.ts`, `web/packages/accounts/services/srp.ts`, `web/packages/accounts/services/recovery-key.ts`, `server/ente/user.go`, and `cli/pkg/sign_in.go`. Ente creates a random master/recovery key, wraps the master key with an Argon2id KEK, cross-wraps recovery/master keys, encrypts an account X25519 private key with the master key, uses the public key for sealed data/token delivery, and replaces key attributes during password changes.
- Ente, [Architecture](https://ente.com/architecture/) (accessed 2026-03-31).
- RFC 9106, [Argon2 Memory-Hard Function](https://www.rfc-editor.org/rfc/rfc9106).
- RFC 9807, [OPAQUE](https://www.rfc-editor.org/info/rfc9807).
- RFC 9180, [Hybrid Public Key Encryption](https://www.rfc-editor.org/rfc/rfc9180).
- Libsodium, [Sealed boxes](https://libsodium.gitbook.io/doc/public-key_cryptography/sealed_boxes).
