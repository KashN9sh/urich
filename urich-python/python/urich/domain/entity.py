"""Entity — identity-bearing object (id)."""


class Entity:
    """Entity: equality by id."""

    def __init__(self, id: str) -> None:
        self.id = id

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Entity):
            return False
        return self.id == other.id

    def __hash__(self) -> int:
        return hash(self.id)
