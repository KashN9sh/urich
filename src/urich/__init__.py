"""
Urich — async DDD framework for microservices.
Application is composed from module objects via app.register(module).
"""
from urich.core import Application, Container, Module, HttpModule, Config

__all__ = [
    "Application",
    "Container",
    "Module",
    "HttpModule",
    "Config",
]
