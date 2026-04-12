"""Authentication middleware - adds another layer to the call chain.

Route handler -> AuthMiddleware -> OrderService -> repositories -> models
This creates 5 layers of call depth.
"""

from ..services import OrderService


class AuthMiddleware:
    """Simulates auth checks before service calls."""

    def __init__(self, order_service: OrderService) -> None:
        self._order_service = order_service
        self._authenticated_users: set[str] = set()

    def login(self, user_id: str) -> None:
        self._authenticated_users.add(user_id)

    def logout(self, user_id: str) -> None:
        self._authenticated_users.discard(user_id)

    def _check_auth(self, user_id: str) -> dict | None:
        if user_id not in self._authenticated_users:
            return {"success": False, "error": "Not authenticated"}
        return None

    def create_order(self, user_id: str) -> dict:
        error = self._check_auth(user_id)
        if error:
            return error
        return self._order_service.create_order(customer_id=user_id)

    def add_item(self, user_id: str, order_id: str, product_id: str, quantity: int) -> dict:
        error = self._check_auth(user_id)
        if error:
            return error
        return self._order_service.add_item(order_id, product_id, quantity)

    def confirm_order(self, user_id: str, order_id: str) -> dict:
        error = self._check_auth(user_id)
        if error:
            return error
        return self._order_service.confirm_order(order_id)

    def cancel_order(self, user_id: str, order_id: str) -> dict:
        error = self._check_auth(user_id)
        if error:
            return error
        return self._order_service.cancel_order(order_id)
