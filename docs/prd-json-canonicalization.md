# PRD: RFC 8785 JSON canonicalization Core module

## Status

Proposed. Do not implement this alongside `core/openweight`; Gradus permits editing one Core module at a time.

## Problem

OpenWeight files are JSON. Equivalent JSON documents can differ in whitespace, object-key order, string escaping, and numeric spelling. Future encrypted blobs may need a stable semantic digest for integrity metadata, signatures, conflict detection, or deduplication, while still retaining the original JSON bytes for unchanged-file recovery.

`serde_json` output is deterministic for current Gradus types but is not a cross-version or cross-platform canonical-JSON contract. It must not be used as a cryptographic canonicalization format.

## Goal

Create an independent Core Rust module, `gradus-json-canonicalization`, that converts an eligible JSON byte sequence into RFC 8785 JSON Canonicalization Scheme (JCS) UTF-8 bytes.

For equivalent eligible JSON values, every conforming implementation must produce identical canonical bytes.

## Non-goals

- Do not parse OpenWeight or depend on `core/openweight`.
- Do not encrypt, decrypt, hash, sign, store, upload, or deduplicate data.
- Do not preserve source whitespace, source key order, or source numeric spelling.
- Do not choose a deterministic encryption algorithm.
- Do not impose file-size, rate, timeout, or upload policies; those belong to the calling boundary.

## Standard and semantic requirements

The implementation must conform to RFC 8785, including its I-JSON constraints:

- Accept raw UTF-8 JSON input only.
- Reject malformed JSON, duplicate object-property names, invalid Unicode/lone surrogates, non-finite numbers, and numbers not representable as IEEE 754 double precision.
- Emit no insignificant whitespace.
- Serialize primitives using the RFC 8785 / ECMAScript rules.
- Sort every object recursively by the raw property names' UTF-16 code-unit ordering. Do not use Rust's Unicode scalar-value or locale ordering as a substitute.
- Preserve array element order while recursively canonicalizing objects inside arrays.
- Emit UTF-8 bytes.

RFC 8785 is the authoritative behavioral specification. RFC test vectors and its numeric/string/property-order examples are mandatory regression inputs.

## Proposed public API

All public types and functions belong in `src/api.rs` and are re-exported from `src/lib.rs`.

```rust
pub fn canonicalize(input: &[u8]) -> Result<Vec<u8>, CanonicalizationError>;

pub enum CanonicalizationError {
    InvalidUtf8,
    InvalidJson { message: String },
    DuplicateProperty { path: String, name: String },
    IneligibleNumber { path: String },
    InvalidUnicode { path: String },
}
```

The exact error representation may be refined during implementation, but it must:

- distinguish malformed input from RFC 8785-ineligible valid JSON;
- identify the JSON location when the parser can do so; and
- implement `Display` and `std::error::Error`.

Do not expose an API taking `serde_json::Value` as the only canonicalization entry point: ordinary value parsing loses duplicate-property information before the canonicalizer can reject it. A value-based convenience API may be added only if its documentation makes that limitation explicit and it is not used for untrusted source JSON.

## Architecture

1. Start from `core/template` in a new `core/json-canonicalization/` crate.
2. Keep parser/canonicalization implementation private under `src/private/`.
3. Parse bytes with duplicate-key detection before conversion to an in-memory representation.
4. Represent object keys in a form that supports RFC 8785 UTF-16 code-unit sorting.
5. Serialize directly to a byte buffer according to RFC 8785; do not rely on ordinary `serde_json` map serialization.
6. Keep the module dependency-flat. A vetted, demonstrably conformant RFC 8785 Rust dependency is acceptable only after reviewing its target support, licensing, duplicate-key behavior, and test-vector coverage.
7. Update `docs/core.mmd` with the new independent module node when implementation starts.

## Untrusted-input requirements

Canonicalization is CPU- and allocation-intensive for large or deeply nested JSON. The module must avoid panics on caller-provided bytes and document that callers enforce maximum input size and request timeouts before invocation.

Implementation must test and document behavior for:

- deeply nested arrays/objects;
- very large property names and strings;
- duplicate keys at nested levels;
- pathological object-key ordering inputs;
- numeric boundary values from RFC 8785 Appendix B; and
- invalid UTF-8 and invalid surrogate escape sequences.

## Test plan

- RFC 8785 examples for literals, strings, numbers, object sorting, nested objects, and arrays.
- RFC Appendix B numeric serialization vectors.
- UTF-16 sort-order vectors, including characters whose Unicode scalar ordering differs from UTF-16 code-unit ordering.
- Duplicate-key rejection tests at root and nested paths.
- I-JSON rejection tests for out-of-range numbers and invalid Unicode.
- Determinism: canonicalize identical input repeatedly and compare bytes.
- Equivalence: differently formatted/key-ordered eligible documents produce the same bytes.
- Non-equivalence: semantically distinct JSON produces distinct canonical bytes in test vectors.
- Wasm compile validation once a binding target is in scope.
- 100% public API line and function coverage through `cargo coverage`.

## Future encryption integration

The encrypted-blob layer should treat canonical JSON as an optional derived representation:

```text
original validated JSON bytes -> encrypted payload
RFC 8785(original JSON) -> optional semantic digest/signature input
```

The original bytes remain the encrypted payload to support unchanged-file recovery. Canonical bytes must not replace them.

If a canonical digest is visible to an untrusted server, it can reveal equality between documents. The future encrypted-blob design must decide whether that digest is encrypted, keyed, or omitted.

## Acceptance criteria

- Output matches RFC 8785 test vectors byte-for-byte.
- Invalid UTF-8, malformed JSON, duplicate keys, invalid Unicode, and I-JSON-ineligible numbers return errors without panicking.
- Equivalent eligible JSON inputs canonicalize to identical bytes.
- Object sorting follows UTF-16 code-unit ordering recursively.
- The module has no OpenWeight, encryption, server, or platform-binding dependency.
- Module README, integration tests, `docs/core.mmd`, `cargo coverage`, and repository `pre-commit` pass when implementation begins.
