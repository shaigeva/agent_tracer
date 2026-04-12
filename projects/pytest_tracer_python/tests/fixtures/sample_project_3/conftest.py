"""
Fixtures for sample_project_3.

Demonstrates a deep call chain: route -> middleware -> service -> repository -> model
"""

import pytest
from src.middleware import AuthMiddleware
from src.models import Product
from src.repositories import OrderRepository, ProductRepository
from src.routes import OrderRoutes
from src.services import OrderService


@pytest.fixture
def product_repo():
    """Product repository pre-loaded with sample products."""
    repo = ProductRepository()
    repo.add(Product(id="prod-1", name="Widget", price=9.99, stock=100))
    repo.add(Product(id="prod-2", name="Gadget", price=24.99, stock=5))
    repo.add(Product(id="prod-3", name="Doohickey", price=49.99, stock=0))
    return repo


@pytest.fixture
def order_repo():
    """Empty order repository."""
    return OrderRepository()


@pytest.fixture
def order_service(order_repo, product_repo):
    """Order service wired to repositories."""
    return OrderService(order_repo, product_repo)


@pytest.fixture
def auth(order_service):
    """Auth middleware wrapping the order service."""
    return AuthMiddleware(order_service)


@pytest.fixture
def routes(auth):
    """Route handlers wrapping auth middleware."""
    return OrderRoutes(auth)


@pytest.fixture
def logged_in_user(auth):
    """A logged-in user ID."""
    user_id = "user-1"
    auth.login(user_id)
    return user_id
