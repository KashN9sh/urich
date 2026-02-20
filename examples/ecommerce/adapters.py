"""
Adapter stubs: implementation by user or separate package.
In a real app: Redis/NATS/Kafka, Postgres outbox, HTTP transport, etc.
"""


class RedisEventAdapter:
    def __init__(self, url: str):
        self.url = url


class PostgresOutboxStorage:
    def __init__(self, database_url: str):
        self.database_url = database_url


class KafkaPublisher:
    def __init__(self, brokers: str):
        self.brokers = brokers


class JsonHttpTransport:
    pass
