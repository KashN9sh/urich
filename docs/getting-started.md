# Getting started

Modules and handlers are fully typed; your IDE will suggest methods and fields and catch type errors.

## Prerequisites

- Python 3.12+
- (Optional) [uv](https://docs.astral.sh/uv/) or `pip` for install

## Install

```bash
pip install urich
# For running the app:
pip install "urich[dev]"
# For CLI code generation:
pip install "urich[cli]"
```

## Minimal app

1. Create a file `main.py`:

```python
from urich import Application
from orders.module import orders_module

app = Application()
app.register(orders_module)
app.openapi(title="My API", version="0.1.0")
```

2. Run with uvicorn:

```bash
uvicorn main:app --reload
```

3. Open [http://localhost:8000/docs](http://localhost:8000/docs) for interactive Swagger UI (no extra config). You'll see routes like:

- `POST /orders/commands/create_order`
- `GET /orders/queries/get_order`

These paths match the implementation: prefix `/orders`, then `/commands/...` or `/queries/...` with endpoint names in snake_case.

The `orders` module in this example is a **DomainModule**: one object that declares aggregate, repository, command, query and event handler. You can scaffold it with the CLI or copy from the [ecommerce example](https://github.com/KashN9sh/urich/tree/main/examples/ecommerce).

## Using the CLI

Recommended sequence: **create-app** → **add-context** → **add-aggregate**. From an empty directory:

```bash
urich create-app myapp
cd myapp
urich add-context orders --dir .
urich add-aggregate orders Order --dir .
```

If the app directory or context already exists, existing files are **not overwritten** by default; use `--force` to overwrite. See [CLI](cli.md) for options and behavior.

Then in `main.py` (or your app entrypoint):

```python
from urich import Application
from orders.module import orders_module

app = Application()
app.register(orders_module)
app.openapi(title="My API", version="0.1.0")
```

Run with `uvicorn main:app --reload` and visit `/docs`.

## Next

- [Application & modules](guide/application.md) — how `Application` and `app.register()` work
- [Domain module](guide/domain-module.md) — structure of a bounded context (domain, application, infrastructure, module.py)
