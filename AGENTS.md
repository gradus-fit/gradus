# Gradus

End to end encrypted monorepo for Gradus Fitness App

## Lego based development

### What is a module
- First level of folders divide each one of the monorepo components: server, core. Start by running `cd` into the relevant folder
- Inside each folder you'll find **module** folders. Only folders within a component are modules; repository-level directories such as `docs` are not modules.
- A module is:
    - A self contained piece of code with a public interface and private code to make it possible to provide the public interface.
    - Plus tests for the public interface and a README teaching how to use the public interface
- The rule of thumb for a module is something focused, small, self contained. Utilities, building blocks for the application
- Dependencies are shared: use the component dependency managment tool that is shared across all modules
- I should be able to copy and paste those modules and use them without too much modifications
- You are allowed on depending on another module to make your module work, but keep them as flat as possible

### Module Layout Conventions

#### Rust
- `src/lib.rs` is the crate entry point and only declares private modules and re-exports the public API.
- `src/api.rs` contains every public type, function, and trait that the crate re-exports.
- `src/private/` contains implementation details and must not be re-exported from `src/lib.rs`.
- `tests/` contains integration tests that import the crate root only; tests must not access `src/api.rs` or `src/private/` directly.
- Start new Core modules by copying `core/template` and replacing its placeholders.

#### Python
- Start new Server modules by copying `server/template` to `server/src/<module>` and replacing every `Template`, `create`, and lowercase `template` placeholder, sample implementation, test, and README content.
- Declare their public interface in the module-root `__init__.py` with a literal `__all__` list or tuple.
- Tests must use `@pytest.mark.public_api("src.<module>", "<export>", ...)` to declare every public export they exercise
- Pytest enforces that every `__all__` export is declared by at least one test.

### Developing new code
- Each module should have 100% test coverage for its public interface.
- You are forbidden to edit two modules at the same time. If I ask you to develop a feature that would require editing two modules:
    - Refuse developing the feature
    - Reply with one or more PRDs: a document with a request that I can use to implement the required changes in the module you're blocked, so you can continue your work
    - PRDs lives in the `docs` folder in repo root
- Always edit the mermaid chart at `docs/<server|core>.mmd` connecting boxes to show module graph dependencies
- If you feel that a feature would need two or more modules to be created from scratch, push back and create PRDs instead

## Components

### Core
- Safe Rust shared modules across all components and platforms
- Target platforms and bindings:
    - Python (server / CLI)
    - Dart bindings for the Flutter app (Android, iOS)
    - Wasm bindings for the Typescript web app
- Platform bindings are separate Core modules. A binding module must expose a uniform public interface for its target platform; Rust domain modules do not need to include bindings directly.
- Responsabilities:
    - Provide operations done for all other components so we don't reimplement the same feature twice
    - End to end encryption / decryption of arbitrary files
    - Parsers for fitness data (Takes client data, returns standarized data. More bellow)
    - Owns the user data Vault schema. More bellow
- User Data Vault Schema:
    - This is how we represent user data on server and on device
    - Clients will call you to generate encrypted blobs ready to be uploaded to the server
    - Server will call you to validated blobs are correct and can be saved
    - Clients will call you to decrypt blobs aswell
    - Given that, you'll own how we'll save data on the server.
    - Read `docs/data_model.md` for the contract
- Standards:
    - Strength training data: we use OpenWeight, schema docs: https://openweight.dev/schema/
    - Outdoor activities: we use Garming fit file format

### Server
- Python, FastAPI, SQLModel, strongly typed server project
- Responsabilities:
    - Provide registrarion, billing, admin features
    - User data Vault: receives and provides user encrypted blobs. This is a E2E encrypted app, server cannot read user data

## Required during development

- Run `pre-commit` at the end of development, to run the full test suite and ensure eveything passes
- When dealing with new features, do web search, make sure you're using the latest starndards of the programming languages and libraries
