# Server Module Template

Copy this directory to create a focused Server module:

```sh
cd server
cp -R template src/my_module
```

Then replace every occurrence of the following placeholders:

- `Template` with the module's public type or types.
- `create` with the module's public operation or operations.
- Lowercase `template` in private and test file names, imports, and test names.
- The sample implementation in `src/_template.py` and `src/models.py` with the module's behavior.
- The sample test in `tests/test_template.py` with tests for every public export.
- This README with module-specific API documentation, including names, arguments, behavior, errors, and examples.

Do not retain the sample implementation or test after copying; they exist only to keep this template type-checkable and public-interface-test verified.

## Required layout

```text
<module>/
├── README.md
├── __init__.py
├── src/
│   ├── __init__.py
│   ├── _template.py
│   └── models.py
└── tests/
    ├── __init__.py
    └── test_template.py
```

- Declare every public type and operation in the module-root `__init__.py` and list them in one literal `__all__` declaration.
- Keep implementation details under `src/`; prefix private modules and functions with an underscore.
- Tests must import the module root only. The template's relative import and `public_api` marker module name adapt automatically after copying.
- Update `../docs/server.mmd` with the module and its dependencies.

## Example public API

The sample API illustrates a typed operation returning an immutable dataclass. Replace it with the new module's domain API.

```python
from src.my_module import Template, create

result = create("example")

assert result.value == "example"
```
