"""
Stateless bounded context: no aggregate, no repository.
Shows DomainModule with only commands and queries (e.g. calculator, validator).
"""
from dataclasses import dataclass
from urich.ddd import DomainModule, Command, Query


@dataclass
class CalculateCommission(Command):
    amount_cents: int
    rate_percent: float


@dataclass
class ValidateAmount(Query):
    amount_cents: int


def calculate_commission_handler(cmd: CalculateCommission) -> int:
    return int(cmd.amount_cents * cmd.rate_percent / 100)


def validate_amount_handler(query: ValidateAmount) -> dict:
    return {"valid": query.amount_cents >= 0, "amount_cents": query.amount_cents}


commission_module = (
    DomainModule("commission")
    .command(CalculateCommission, calculate_commission_handler)
    .query(ValidateAmount, validate_amount_handler)
)
