"""
Scenario tests for the full order flow through all 5 layers.

Call chain: test -> routes -> middleware -> service -> repository -> model
Each test exercises code across 6+ source files.
"""

import pytest


@pytest.mark.scenario
@pytest.mark.behavior("order-management")
@pytest.mark.behavior("full-stack")
def test_create_order_via_route(routes, logged_in_user):
    """
    Create order through the full route->model stack

    GIVEN an authenticated user
    WHEN creating an order via the route handler
    THEN order is created through all layers
    """
    result = routes.post_order(logged_in_user)

    assert result["success"] is True
    assert result["order"]["customer_id"] == logged_in_user
    assert result["order"]["status"] == "draft"


@pytest.mark.scenario
@pytest.mark.behavior("order-management")
@pytest.mark.behavior("full-stack")
def test_add_item_to_order(routes, logged_in_user):
    """
    Add item through route->middleware->service->repo->model chain

    GIVEN an authenticated user with a draft order
    WHEN adding a product via the route handler
    THEN product stock is reserved and item appears on order
    """
    order_result = routes.post_order(logged_in_user)
    order_id = order_result["order"]["id"]

    result = routes.post_order_item(logged_in_user, order_id, "prod-1", 2)

    assert result["success"] is True
    assert len(result["order"]["items"]) == 1
    assert result["order"]["items"][0]["product_name"] == "Widget"
    assert result["order"]["items"][0]["quantity"] == 2
    assert result["order"]["total"] == 19.98


@pytest.mark.scenario
@pytest.mark.behavior("order-management")
@pytest.mark.behavior("full-stack")
def test_complete_order_lifecycle(routes, logged_in_user):
    """
    Full order lifecycle: create -> add items -> confirm -> ship

    GIVEN an authenticated user
    WHEN going through the complete order flow
    THEN each step transitions through all layers correctly
    """
    # Create order
    order_result = routes.post_order(logged_in_user)
    order_id = order_result["order"]["id"]

    # Add items (touches product model stock checking)
    routes.post_order_item(logged_in_user, order_id, "prod-1", 1)
    routes.post_order_item(logged_in_user, order_id, "prod-2", 2)

    # Confirm (touches order model state machine)
    confirm_result = routes.post_order_confirm(logged_in_user, order_id)
    assert confirm_result["success"] is True
    assert confirm_result["order"]["status"] == "confirmed"


@pytest.mark.scenario
@pytest.mark.behavior("order-management")
@pytest.mark.behavior("cancellation")
@pytest.mark.behavior("full-stack")
def test_cancel_order_releases_stock(routes, logged_in_user, product_repo):
    """
    Cancelling an order releases reserved product stock

    GIVEN an order with items (stock was reserved)
    WHEN cancelling the order via route handler
    THEN stock is released back through all layers
    """
    order_result = routes.post_order(logged_in_user)
    order_id = order_result["order"]["id"]

    # Reserve 3 widgets (stock goes from 100 to 97)
    routes.post_order_item(logged_in_user, order_id, "prod-1", 3)
    product = product_repo.get("prod-1")
    assert product.stock == 97

    # Cancel releases stock (back to 100)
    cancel_result = routes.post_order_cancel(logged_in_user, order_id)
    assert cancel_result["success"] is True
    assert cancel_result["order"]["status"] == "cancelled"
    assert product.stock == 100


@pytest.mark.scenario
@pytest.mark.behavior("authentication")
@pytest.mark.behavior("full-stack")
@pytest.mark.error
def test_unauthenticated_create_order(routes):
    """Route rejects unauthenticated order creation"""
    result = routes.post_order("unknown-user")

    assert result["success"] is False
    assert "not authenticated" in result["error"].lower()


@pytest.mark.scenario
@pytest.mark.behavior("order-management")
@pytest.mark.behavior("validation")
@pytest.mark.behavior("full-stack")
@pytest.mark.error
def test_add_out_of_stock_item(routes, logged_in_user):
    """
    Cannot add item when product is out of stock

    GIVEN a product with zero stock
    WHEN trying to add it to an order
    THEN error propagates from model through all layers
    """
    order_result = routes.post_order(logged_in_user)
    order_id = order_result["order"]["id"]

    # prod-3 has stock=0
    result = routes.post_order_item(logged_in_user, order_id, "prod-3", 1)

    assert result["success"] is False
    assert "stock" in result["error"].lower()


@pytest.mark.scenario
@pytest.mark.behavior("order-management")
@pytest.mark.behavior("validation")
@pytest.mark.error
def test_confirm_empty_order(routes, logged_in_user):
    """Cannot confirm order with no items"""
    order_result = routes.post_order(logged_in_user)
    order_id = order_result["order"]["id"]

    result = routes.post_order_confirm(logged_in_user, order_id)

    assert result["success"] is False
    assert "no items" in result["error"].lower()
