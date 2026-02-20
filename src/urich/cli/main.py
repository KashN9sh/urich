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


@app.command()
def create_app(
    name: str = typer.Argument(..., help="App name (folder)"),
    directory: Path = typer.Option(Path("."), "--dir", "-d", help="Parent directory"),
) -> None:
    """Create app skeleton: folder, main.py, config.py."""
    _ensure_typer()
    root = directory / name
    root.mkdir(parents=True, exist_ok=True)
    (root / "main.py").write_text(T.MAIN_PY.format(first_context="orders"), encoding="utf-8")
    (root / "config.py").write_text(
        '"""Config (env/file)."""\n# from dataclasses import dataclass\n# settings = ...\n',
        encoding="utf-8",
    )
    typer.echo(f"Created: {root}/ (main.py, config.py). Add context: urich add-context <name> --dir {root}")


@app.command()
def add_context(
    name: str = typer.Argument(..., help="Bounded context name (e.g. orders)"),
    directory: Path = typer.Option(Path("."), "--dir", "-d", help="App root directory"),
) -> None:
    """Add bounded context: folder with domain, application, infrastructure, module (skeleton)."""
    _ensure_typer()
    ctx_dir = directory / name
    ctx_dir.mkdir(parents=True, exist_ok=True)
    ctx_dir.joinpath("domain.py").write_text(
        T.CONTEXT_SKELETON.format(context=name),
        encoding="utf-8",
    )
    ctx_dir.joinpath("application.py").write_text(
        T.CONTEXT_APPLICATION_SKELETON.format(context=name),
        encoding="utf-8",
    )
    ctx_dir.joinpath("infrastructure.py").write_text(
        T.CONTEXT_INFRASTRUCTURE_SKELETON.format(context=name),
        encoding="utf-8",
    )
    ctx_dir.joinpath("module.py").write_text(
        T.CONTEXT_MODULE_SKELETON.format(context=name),
        encoding="utf-8",
    )
    typer.echo(f"Context «{name}»: {ctx_dir}/. Add aggregate: urich add-aggregate {name} <AggregateName> --dir {directory}")


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
    ctx_dir.joinpath("domain.py").write_text(
        T.DOMAIN_PY.format(context=context, aggregate=aggregate, aggregate_lower=agg_lower),
        encoding="utf-8",
    )
    ctx_dir.joinpath("application.py").write_text(
        T.APPLICATION_PY.format(context=context, aggregate=aggregate, aggregate_lower=agg_lower),
        encoding="utf-8",
    )
    ctx_dir.joinpath("infrastructure.py").write_text(
        T.INFRASTRUCTURE_PY.format(context=context, aggregate=aggregate, aggregate_lower=agg_lower),
        encoding="utf-8",
    )
    ctx_dir.joinpath("module.py").write_text(
        T.MODULE_PY.format(context=context, aggregate=aggregate, aggregate_lower=agg_lower),
        encoding="utf-8",
    )
    typer.echo(f"Aggregate «{aggregate}» in «{context}»: {ctx_dir}/. In main.py: from {context}.module import {context}_module; app.register({context}_module)")


def main() -> None:
    """Entry point for the urich console command."""
    app()


if __name__ == "__main__":
    main()
