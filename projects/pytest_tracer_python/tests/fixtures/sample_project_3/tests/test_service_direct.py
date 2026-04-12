"""
Scenario tests that exercise the service layer directly (bypassing routes/middleware).

This tests 3-layer depth: service -> repository -> model.
Useful for comparing coverage breadth with the full-stack tests.
"""

import pytest


@pytest.mark.scenario
@pytest.mark.behavior("order-management")
def test_create_order_direct(order_service):
    """Create order directly through service"""
    result = order_service.create_order("customer-1")

    assert result["success"] is True
    assert result["order"]["status"] == "draft"


@pytest.mark.scenario
@pytest.mark.behavior("order-management")
def test_add_item_direct(order_service):
    """Add item to order through service layer"""
    order_result = order_service.create_order("customer-1")
    order_id = order_result["order"]["id"]

    result = order_service.add_item(order_id, "prod-1", 5)

    assert result["success"] is True
    assert result["order"]["total"] == 49.95


@pytest.mark.scenario
@pytest.mark.behavior("order-management")
@pytest.mark.error
def test_add_item_nonexistent_product(order_service):
    """Adding nonexistent product returns error"""
    order_result = order_service.create_order("customer-1")
    order_id = order_result["order"]["id"]

    result = order_service.add_item(order_id, "prod-nonexistent", 1)

    assert result["success"] is False
    assert "not found" in result["error"].lower()


@pytest.mark.scenario
@pytest.mark.behavior("order-management")
@pytest.mark.behavior("listing")
def test_list_customer_orders(order_service):
    """List orders for a specific customer"""
    order_service.create_order("customer-1")
    order_service.create_order("customer-1")
    order_service.create_order("customer-2")

    result = order_service.list_customer_orders("customer-1")

    assert result["success"] is True
    assert result["count"] == 2
