"""Command and query — CQRS markers."""
from dataclasses import dataclass


@dataclass
class Command:
    """Command: intent to change state. One handler per command type."""
    pass


@dataclass
class Query:
    """Query: intent to read. One handler per query type."""
    pass
