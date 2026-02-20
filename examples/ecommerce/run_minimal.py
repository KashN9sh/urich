"""
Minimal run: Application + orders DomainModule only.
To run: uvicorn run_minimal:app --reload
Then open http://localhost:8000/docs for Swagger UI.
"""
import sys
from pathlib import Path

# example lives in examples/ecommerce
sys.path.insert(0, str(Path(__file__).resolve().parent))

from urich import Application
from orders.module import orders_module

app = Application()
app.register(orders_module)
app.openapi(title="Ecommerce API", version="0.1.0")
