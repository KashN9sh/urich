# Stateless context (no persistence)

A **DomainModule** can have only commands and queries, with no aggregate and no repository. Use this for bounded contexts that do not store state in a database: calculators, validators, external API gateways, etc.

## Example

```python
from dataclasses import dataclass
from urich.ddd import DomainModule, Command, Query


@dataclass
class CalculateCommission(Command):
    amount_cents: int
    rate_percent: float


@dataclass
class ValidateRule(Query):
    rule_id: str
    payload: dict


def calculate_commission_handler(cmd: CalculateCommission) -> int:
    return int(cmd.amount_cents * cmd.rate_percent / 100)


def validate_rule_handler(query: ValidateRule) -> dict:
    # Your validation logic; no repository needed
    return {"valid": True, "rule_id": query.rule_id}


# Module with no .aggregate() and no .repository()
commission_module = (
    DomainModule("commission")
    .command(CalculateCommission, calculate_commission_handler)
    .query(ValidateRule, validate_rule_handler)
)

app.register(commission_module)
```

Handlers receive only the command/query (and any dependencies you register via `.bind()`). No repository or EventBus is required unless you add `.repository()` or `.on_event()`.
