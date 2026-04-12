"""Route handlers - the outermost layer of the call chain.

Full call depth: route -> middleware -> service -> repository -> model
This is 5 layers deep, validating deep tracing works correctly.
"""

from ..middleware import AuthMiddleware


class OrderRoutes:
    """Simulates HTTP route handlers."""

    def __init__(self, auth: AuthMiddleware) -> None:
        self._auth = auth

    def post_order(self, user_id: str) -> dict:
        """POST /orders - Create a new order."""
        return self._auth.create_order(user_id)

    def post_order_item(self, user_id: str, order_id: str, product_id: str, quantity: int) -> dict:
        """POST /orders/:id/items - Add item to order."""
        return self._auth.add_item(user_id, order_id, product_id, quantity)

    def post_order_confirm(self, user_id: str, order_id: str) -> dict:
        """POST /orders/:id/confirm - Confirm an order."""
        return self._auth.confirm_order(user_id, order_id)

    def post_order_cancel(self, user_id: str, order_id: str) -> dict:
        """POST /orders/:id/cancel - Cancel an order."""
        return self._auth.cancel_order(user_id, order_id)
