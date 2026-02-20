"""Repository — interface for aggregate persistence."""
from abc import ABC, abstractmethod
from typing import Generic, Optional, TypeVar

T = TypeVar("T")


class Repository(ABC, Generic[T]):
    """Repository interface: get by id, add new, save existing."""

    @abstractmethod
    async def get(self, id: str) -> Optional[T]:
        ...

    @abstractmethod
    async def add(self, aggregate: T) -> None:
        ...

    @abstractmethod
    async def save(self, aggregate: T) -> None:
        ...
