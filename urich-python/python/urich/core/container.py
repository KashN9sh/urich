"""Minimal DI container: register by type/protocol, resolve dependencies."""
from __future__ import annotations

import inspect
from typing import Any, Callable, TypeVar

T = TypeVar("T")


def _resolve_annotation(ann: str, cls: type[Any]) -> Any:
    """Resolve a string annotation (from __future__ annotations) to the actual class."""
    import sys
    mod = sys.modules.get(cls.__module__)
    if mod is not None and hasattr(mod, ann):
        return getattr(mod, ann)
    return ann


def _instantiate_with_container(container: Container, cls: type[T]) -> T:
    """Create an instance of cls, resolving __init__ dependencies from the container."""
    sig = inspect.signature(cls)
    kwargs: dict[str, Any] = {}
    for name, param in sig.parameters.items():
        if name == "self" or param.annotation is inspect.Parameter.empty:
            continue
        ann = param.annotation
        if isinstance(ann, str):
            ann = _resolve_annotation(ann, cls)
        kwargs[name] = container.resolve(ann)
    return cls(**kwargs)


class Container:
    """
    Register by type (or key) and resolve via factory.
    Allows registering an implementation for a protocol/abstraction.
    """

    def __init__(self) -> None:
        self._registry: dict[type[Any] | str, Callable[[], Any]] = {}
        self._singletons: dict[type[Any] | str, Any] = {}
        self._singleton_keys: set[type[Any] | str] = set()

    def register(self, key: type[T] | type[Any] | str, factory: Callable[[], T], singleton: bool = True) -> None:
        """Register a factory for a type or string key."""
        self._registry[key] = factory
        if singleton:
            self._singleton_keys.add(key)
            self._singletons[key] = None  # placeholder until first resolve

    def register_instance(self, key: type[T] | type[Any] | str, instance: T) -> None:
        """Register a ready-made instance."""
        self._registry[key] = lambda: instance
        self._singletons[key] = instance
        self._singleton_keys.add(key)

    def resolve(self, key: type[T] | type[Any] | str) -> T:
        """Resolve an instance by type or key."""
        if key not in self._registry:
            raise KeyError(f"No registration for {key}")
        if key in self._singleton_keys and self._singletons.get(key) is not None:
            return self._singletons[key]
        instance = self._registry[key]()
        if key in self._singleton_keys:
            self._singletons[key] = instance
        return instance

    def register_class(self, cls: type[T], singleton: bool = True) -> None:
        """Register a class: on resolve an instance is created with dependencies from the container."""
        self.register(key=cls, factory=lambda: _instantiate_with_container(self, cls), singleton=singleton)
