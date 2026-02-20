# CLI

Install the CLI extra:

```bash
pip install "urich[cli]"
```

Commands are available as `urich <command>`.

## create-app

Scaffold a new application directory.

```bash
urich create-app myapp
cd myapp
```

Creates a minimal layout with `main.py` and a placeholder for registering modules.

## add-context

Add a bounded context (folder with domain, application, infrastructure, module.py skeleton).

```bash
urich add-context orders --dir .
```

Use `--dir` to specify the project root (default is current directory). Creates e.g. `orders/domain.py`, `orders/application.py`, `orders/infrastructure.py`, `orders/module.py`.

## add-aggregate

Add an aggregate to an existing context.

```bash
urich add-aggregate orders Order --dir .
```

Creates or updates the orders context with an aggregate named `Order` and the usual domain/application/infrastructure pieces.

## After scaffolding

In your app entrypoint (e.g. `main.py`):

```python
from urich import Application
from orders.module import orders_module

app = Application()
app.register(orders_module)
app.openapi(title="My API", version="0.1.0")
```

Run with:

```bash
uvicorn main:app --reload
```

Then open [http://localhost:8000/docs](http://localhost:8000/docs).
