"""Single config object: user passes it when creating the app; available via DI."""
import os
from typing import Any


class Config:
    """
    Application config. User creates their own class or instance
    and passes to Application(config=...); then available via container.resolve(Config).
    """

    @classmethod
    def load_from_env(cls, prefix: str = "APP_", **defaults: Any) -> dict[str, Any]:
        """Load from os.environ with prefix and defaults. Returns dict for MyConfig(**Config.load_from_env())."""
        result = dict(defaults)
        for key, value in os.environ.items():
            if key.startswith(prefix):
                name = key[len(prefix):].lower()
                result[name] = value
        return result
