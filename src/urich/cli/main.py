"""
CLI for prototyping: create-app, add-context, add-aggregate.
Generated code composes a DomainModule and registers via app.register(module).
"""
from pathlib import Path

try:
    import typer
except ImportError:
    typer = None  # type: ignore

from urich.cli import templates as T

app = typer.Typer(help="Urich CLI: create app and bounded context.")


def _ensure_typer():
    if typer is None:
        raise SystemExit("CLI requires typer: pip install 'urich[cli]' or pip install typer")


def _snake(name: str) -> str:
    return "".join("_" + c.lower() if c.isupper() else c for c in name).lstrip("_")


_CONFIG_PY = '"""Config (env/file)."""\n# from dataclasses import dataclass\n# settings = ...\n'


@app.command()
def create_app(
    name: str = typer.Argument(..., help="App name (folder)"),
    directory: Path = typer.Option(Path("."), "--dir", "-d", help="Parent directory"),
    force: bool = typer.Option(False, "--force", "-f", help="Overwrite existing main.py and config.py"),
) -> None:
    """Create app skeleton: folder, main.py, config.py."""
    _ensure_typer()
    root = directory / name
    root.mkdir(parents=True, exist_ok=True)

    skipped = []
    for filename, content in [
        ("main.py", T.MAIN_PY.format(first_context="orders")),
        ("config.py", _CONFIG_PY),
    ]:
        path = root / filename
        if path.exists() and not force:
            skipped.append(filename)
            typer.echo(f"{filename} already exists, skipped")
        else:
            path.write_text(content, encoding="utf-8")

    hint = f"Add context: urich add-context <context_name> --dir {root}"
    if skipped:
        typer.echo(f"Use --force to overwrite existing files. {hint}")
    else:
        typer.echo(f"Created: {root}/ (main.py, config.py). {hint}")


_CONTEXT_FILES = [
    ("domain.py", T.CONTEXT_SKELETON),
    ("application.py", T.CONTEXT_APPLICATION_SKELETON),
    ("infrastructure.py", T.CONTEXT_INFRASTRUCTURE_SKELETON),
    ("module.py", T.CONTEXT_MODULE_SKELETON),
]


@app.command()
def add_context(
    name: str = typer.Argument(..., help="Bounded context name (e.g. orders)"),
    directory: Path = typer.Option(Path("."), "--dir", "-d", help="App root directory"),
    force: bool = typer.Option(False, "--force", "-f", help="Overwrite existing context files"),
) -> None:
    """Add bounded context: folder with domain, application, infrastructure, module (skeleton)."""
    _ensure_typer()
    ctx_dir = directory / name
    ctx_dir.mkdir(parents=True, exist_ok=True)

    skipped = []
    for filename, template in _CONTEXT_FILES:
        path = ctx_dir / filename
        if not path.exists() or force:
            path.write_text(template.format(context=name), encoding="utf-8")
        else:
            skipped.append(filename)

    if skipped:
        typer.echo(f"Context «{name}» already exists (some files skipped). Use --force to overwrite.")
    typer.echo(f"Add aggregate: urich add-aggregate {name} <AggregateName> --dir {directory}")


def _context_has_aggregates(ctx_dir: Path) -> bool:
    """True if module.py contains an actual .aggregate( call (not in a comment)."""
    module_py = ctx_dir / "module.py"
    if not module_py.exists():
        return False
    text = module_py.read_text(encoding="utf-8")
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("#"):
            continue
        if ".aggregate(" in stripped:
            return True
    return False


@app.command()
def add_aggregate(
    context: str = typer.Argument(..., help="Context name (folder)"),
    aggregate: str = typer.Argument(..., help="Aggregate name (PascalCase, e.g. Order)"),
    directory: Path = typer.Option(Path("."), "--dir", "-d", help="App root directory"),
) -> None:
    """Add aggregate to context: domain, application, infrastructure, module (DomainModule with command/query)."""
    _ensure_typer()
    ctx_dir = directory / context
    if not ctx_dir.is_dir():
        typer.echo(f"Context folder not found: {ctx_dir}. Run first: urich add-context {context} --dir {directory}", err=True)
        raise typer.Exit(1)
    agg_lower = _snake(aggregate)
    fmt = {"context": context, "aggregate": aggregate, "aggregate_lower": agg_lower}

    if not _context_has_aggregates(ctx_dir):
        # First aggregate: full write of all four files.
        ctx_dir.joinpath("domain.py").write_text(
            T.DOMAIN_PY.format(**fmt),
            encoding="utf-8",
        )
        ctx_dir.joinpath("application.py").write_text(
            T.APPLICATION_PY.format(**fmt),
            encoding="utf-8",
        )
        ctx_dir.joinpath("infrastructure.py").write_text(
            T.INFRASTRUCTURE_PY.format(**fmt),
            encoding="utf-8",
        )
        ctx_dir.joinpath("module.py").write_text(
            T.MODULE_PY.format(**fmt),
            encoding="utf-8",
        )
    else:
        # Second and subsequent aggregates: append blocks to existing files.
        for filename, template in [
            ("domain.py", T.DOMAIN_PY_APPEND),
            ("application.py", T.APPLICATION_PY_APPEND),
            ("infrastructure.py", T.INFRASTRUCTURE_PY_APPEND),
            ("module.py", T.MODULE_PY_APPEND),
        ]:
            path = ctx_dir / filename
            existing = path.read_text(encoding="utf-8") if path.exists() else ""
            path.write_text(existing + template.format(**fmt), encoding="utf-8")

    typer.echo(f"Aggregate «{aggregate}» in «{context}»: {ctx_dir}/. In main.py: from {context}.module import {context}_module; app.register({context}_module)")


def main() -> None:
    """Entry point for the urich console command."""
    app()


if __name__ == "__main__":
    main()
