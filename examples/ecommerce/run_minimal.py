"""
Minimal run: Application + orders DomainModule only.
To run: uvicorn run_minimal:app.starlette --reload
Or: python -c "from run_minimal import app; import uvicorn; uvicorn.run(app.starlette)"
"""
import sys
from pathlib import Path

# example lives in examples/ecommerce
sys.path.insert(0, str(Path(__file__).resolve().parent))

from urich import Application
from orders.module import orders_module

app = Application()
app.register(orders_module)
