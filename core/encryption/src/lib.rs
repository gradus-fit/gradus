// SPDX-License-Identifier: AGPL-3.0-or-later

//! Versioned envelope encryption for Gradus vault blobs.

mod api;
mod private;

pub use api::{
    BlobBinding, EncryptedBlob, EncryptionError, PersonalVaultKey, decode_encrypted_blob,
    decode_personal_vault_key_from_storage, decrypt_blob, encode_encrypted_blob,
    encode_personal_vault_key_for_storage, encrypt_blob, generate_personal_vault_key,
};
