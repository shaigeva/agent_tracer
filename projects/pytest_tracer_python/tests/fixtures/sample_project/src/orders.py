"""
Sample orders module for testing pytest-tracer.

This module provides order management functions that are exercised
by the scenario tests in tests/test_orders.py.
"""


def create_order(user_id: str, items: list[dict]) -> dict:
    """
    Create a new order for a user.

    Args:
        user_id: ID of the user placing the order
        items: List of items with product_id and quantity

    Returns:
        dict with order details or error
    """
    if not user_id:
        return {"status": 400, "error": "User ID required"}

    if not items:
        return {"status": 400, "error": "Order must have at least one item"}

    # Calculate total
    total = sum(item.get("quantity", 1) * 10.0 for item in items)

    return {
        "status": 201,
        "order": {
            "id": "order-123",
            "user_id": user_id,
            "items": items,
            "total": total,
            "status": "pending",
        },
    }


def get_order(order_id: str) -> dict:
    """
    Get order by ID.

    Args:
        order_id: Order ID to retrieve

    Returns:
        dict with order details or error
    """
    if order_id == "order-123":
        return {
            "status": 200,
            "order": {
                "id": "order-123",
                "user_id": "user-1",
                "items": [{"product_id": "prod-1", "quantity": 2}],
                "total": 20.0,
                "status": "pending",
            },
        }

    return {"status": 404, "error": "Order not found"}
