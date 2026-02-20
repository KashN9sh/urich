"""ValueObject — value without identity; equality by fields."""
from dataclasses import dataclass


@dataclass(frozen=True)
class ValueObject:
    """Value object: equality by all fields (via dataclass)."""
    pass
