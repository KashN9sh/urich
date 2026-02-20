"""Config (env/file) — user code."""
from dataclasses import dataclass
import os


@dataclass
class Settings:
    redis_url: str
    database_url: str
    kafka_brokers: str
    orders_service_url: str
    notifications_service_url: str


settings = Settings(
    redis_url=os.getenv("REDIS_URL", "redis://localhost:6379"),
    database_url=os.getenv("DATABASE_URL", "postgresql://localhost/ecommerce"),
    kafka_brokers=os.getenv("KAFKA_BROKERS", "localhost:9092"),
    orders_service_url=os.getenv("ORDERS_SERVICE_URL", "http://localhost:8001"),
    notifications_service_url=os.getenv("NOTIFICATIONS_SERVICE_URL", "http://localhost:8002"),
)
