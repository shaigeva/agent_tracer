"""
Scenario tests for order management functionality.

These tests demonstrate the pytest-tracer markers and are used
to test the scenario collector.
"""

import pytest
from src.orders import create_order, get_order


@pytest.mark.scenario
@pytest.mark.behavior("orders")
@pytest.mark.behavior("checkout")
def test_create_order():
    """
    User creates a new order

    GIVEN a logged-in user
    WHEN they submit an order with items
    THEN an order is created with pending status
    """
    result = create_order(user_id="user-1", items=[{"product_id": "prod-1", "quantity": 2}])
    assert result["status"] == 201
    assert result["order"]["status"] == "pending"
    assert result["order"]["total"] == 20.0


@pytest.mark.scenario
@pytest.mark.behavior("orders")
@pytest.mark.error
def test_create_order_empty_items():
    """Order creation fails without items"""
    result = create_order(user_id="user-1", items=[])
    assert result["status"] == 400
    assert "at least one item" in result["error"]


@pytest.mark.scenario
@pytest.mark.behavior("orders")
def test_get_order():
    """User retrieves an existing order"""
    result = get_order("order-123")
    assert result["status"] == 200
    assert result["order"]["id"] == "order-123"


@pytest.mark.scenario
@pytest.mark.behavior("orders")
@pytest.mark.error
def test_get_order_not_found():
    """
    Order retrieval fails for non-existent order

    GIVEN an order ID that does not exist
    WHEN user requests the order
    THEN they receive a not found error
    """
    result = get_order("nonexistent-order")
    assert result["status"] == 404
