# urich-python (PyO3 bindings to urich-core)

This crate provides Python bindings for `urich-core`. It is not built by `cargo build` at the workspace root (the extension must link against Python). Build the Python wheel or install in development with **maturin**:

```bash
# From repo root, with maturin installed (pip install maturin):
maturin develop -m urich-python/Cargo.toml
# Or build a wheel:
maturin build -m urich-python/Cargo.toml
```

Python API (minimal): `CoreApp()` → `register_route(method, path, request_schema=None)`, `set_handler(callable)`, `handle_request(method, path, body)`, `openapi_spec(title, version)`.
