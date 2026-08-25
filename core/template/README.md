# Core Module Template

Copy this directory to create a new Gradus Core Rust module:

```sh
cp -R core/template core/<module>
```

Then replace every occurrence of the following placeholders:

- `gradus-module-template` with the package name.
- `gradus_module_template` with the Rust crate name.
- `Template` and `create` with the module's public types and operations.
- The sample implementation in `src/api.rs` and `src/private/mod.rs` with the module's behavior.
- The sample integration test in `tests/template.rs` with tests for every public export.
- This README with module-specific usage documentation.

Do not retain the sample implementation or test after copying; they exist only to keep this template buildable and coverage-verified.

## Required layout

```text
<module>/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── api.rs
│   └── private/
│       └── mod.rs
└── tests/
```

- Keep every exported type, function, and trait in `src/api.rs`.
- Keep implementation details in `src/private/`.
- Re-export the entire public interface from `src/lib.rs`.
- Test the crate root only from `tests/`.

## Verification

From `core/`, run:

```sh
cargo coverage
```

The workspace discovers immediate child crates automatically and requires 100% line and function coverage for public API source files.
