"""Single config object: user passes it when creating the app; available via DI."""
from typing import Any


class Config:
    """
    Application config. User creates their own class or instance
    and passes to Application(config=...); then available via container.resolve(Config).
    """
    pass


def load_config_from_env(prefix: str = "APP_", **defaults: Any) -> dict[str, Any]:
    """Helper: load from os.environ with prefix and defaults."""
    import os
    result = dict(defaults)
    for key, value in os.environ.items():
        if key.startswith(prefix):
            name = key[len(prefix):].lower()
            result[name] = value
    return result
