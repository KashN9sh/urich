# Domain without Urich

Your domain layer can be **completely free of Urich imports**. Aggregates and events are plain Python types. The framework does **not** require any convention on the aggregate (no special method names). Event publishing is done **in the command handler**: the handler gets events from the aggregate in whatever way the domain provides and publishes them via EventBus.

## No framework contract on the aggregate

The aggregate can have **any shape** (e.g. a dataclass with `id` and fields). The framework does not look at the aggregate result. The handler receives EventBus via constructor injection, loads/saves the aggregate, then publishes domain events explicitly (e.g. after `repo.add()` or `repo.save()`).

Domain events can be any type (e.g. dataclasses); they do not need to inherit from Urich's `DomainEvent`.

## Example: domain with no Urich imports

```python
# myapp/domain/employees.py
from dataclasses import dataclass


@dataclass
class EmployeeCreated:
    employee_id: str
    name: str
    role: str


@dataclass
class Employee:
    id: str
    name: str
    role: str

    @classmethod
    def from_db(cls, id: str, name: str, role: str) -> "Employee":
        return cls(id=id, name=name, role=role)
```

## Module and handler

Register the aggregate and the handler. The handler is responsible for publishing domain events.

```python
DomainModule("employees")
    .aggregate(Employee)
    .repository(IEmployeeRepository, EmployeeRepositoryImpl)
    .command(CreateEmployee, CreateEmployeeHandler)
    .on_event(EmployeeCreated, on_employee_created)
```

The **command handler** injects EventBus and publishes events after saving the aggregate:

```python
class CreateEmployeeHandler:
    def __init__(self, employee_repository: IEmployeeRepository, event_bus: EventBus):
        self._repo = employee_repository
        self._event_bus = event_bus

    async def __call__(self, cmd: CreateEmployee) -> str:
        employee = Employee(id=cmd.employee_id, name=cmd.name, role=cmd.role)
        await self._repo.add(employee)
        await self._event_bus.publish(EmployeeCreated(employee_id=employee.id, name=employee.name, role=employee.role))
        return employee.id
```

The framework returns the handler result in the HTTP response (`result.id` here). It does not inspect the aggregate or publish events itself.
