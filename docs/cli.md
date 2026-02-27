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

- **`--dir`** (or `-d`) — Parent directory for the new app folder (default: current directory).
- **`--force`** (or `-f`) — Overwrite existing `main.py` and `config.py`. By default, existing files are **not overwritten**; the command only creates missing files and prints a hint for skipped ones.

Creates a minimal layout with `main.py` and `config.py`. After running, the CLI prints a hint for the next step (e.g. add a context with `urich add-context <name> --dir <app_root>`).

## add-context

Add a bounded context (folder with domain, application, infrastructure, module skeleton).

```bash
urich add-context orders --dir .
```

- **`--dir`** — App root directory (default: current directory). The context is created as `<dir>/<context_name>/`.
- **`--force`** — Overwrite existing context files. If the context folder already exists, existing files are **not overwritten** without `--force`; a hint is printed.

Creates four files: `domain.py`, `application.py`, `infrastructure.py`, `module.py`. The **module** is a skeleton: a single `DomainModule("{context}")` instance with no `.aggregate()`, `.command()`, etc. Add aggregates with `add-aggregate` (see [Domain module](guide/domain-module.md)).

## add-aggregate

Add an aggregate to an existing context.

```bash
urich add-aggregate orders Order --dir .
```

- **`--dir`** — App root directory (default: current directory). The context is expected at `<dir>/<context_name>/`.

**Behavior:**
- **First aggregate** in the context: all four files (`domain.py`, `application.py`, `infrastructure.py`, `module.py`) are created or fully written with the aggregate, commands, queries, repository and event handler.
- **Second and subsequent aggregates**: files are **appended** with new types and handlers; existing code is not removed.

If the context folder does not exist, the command exits with an error and a hint to run `urich add-context <context> --dir <directory>` first. For the relation to `DomainModule` and multiple `.aggregate()` calls, see [Domain module](guide/domain-module.md).

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
