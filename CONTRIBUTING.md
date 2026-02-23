# Contributing to Urich

Contributions are welcome: bug reports, docs fixes, new features, or ideas.

We aim to respond to issues and pull requests within 1–2 working days. If a reply is delayed, we’ll add a short comment to say so (e.g. “Thanks, will look at this next week”).

## How to contribute

1. **Issues** — Open an issue for a bug or feature idea. Check existing issues first.
2. **Fork & branch** — Fork the repo, create a branch (`fix/...` or `feat/...`).
3. **Code** — Install dev deps: `pip install -e ".[dev]"`. Run tests: `pytest`.
4. **Docs** — For doc changes: `pip install -e ".[docs]"` then `mkdocs serve`.
5. **PR** — Open a pull request against `main`. Describe what and why.

## Development setup

```bash
git clone https://github.com/KashN9sh/urich.git
cd urich
pip install -e ".[dev,docs,cli]"
pytest
mkdocs serve   # docs at http://127.0.0.1:8000
```

## What we welcome

- Bug fixes and documentation improvements
- New adapters (e.g. EventBus over Redis/NATS, Discovery over Consul) as optional packages or examples
- Small, focused PRs are easier to review
- If you’re unsure, open an issue first to discuss

## Code style

- Python 3.12+. Use type hints where it helps.
- No formal style guide; match the existing code in the project.
