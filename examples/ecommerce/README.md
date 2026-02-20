# Example: ecommerce

Target shape of an app on the framework. Code is oriented to the **goal** (how we want to write), not the current implementation — the `urich` package is in development.

## Idea

- **main.py** — app is composed only from `app.register(module)`: domain modules, EventBus, Outbox, Discovery, RPC. Everything described as objects.
- **orders/** — one bounded context: domain (aggregate, events), application (commands/queries, handlers), infrastructure (repository). In **module.py** one `DomainModule` object describes the whole context and is registered in the app.

Run is possible once the framework core is implemented (see root TODO.md).
