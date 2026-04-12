"""Order service orchestrating repositories and models.

Call chain: OrderService -> OrderRepository + ProductRepository -> Order + Product models
This creates 4+ layers of call depth for tracing validation.
"""

from ..models import Order, OrderItem
from ..repositories import OrderRepository, ProductRepository


class OrderService:
    def __init__(self, order_repo: OrderRepository, product_repo: ProductRepository) -> None:
        self._order_repo = order_repo
        self._product_repo = product_repo

    def create_order(self, customer_id: str) -> dict:
        """Create a new empty draft order."""
        order_id = self._order_repo.next_id()
        order = Order(id=order_id, customer_id=customer_id)
        self._order_repo.save(order)
        return {"success": True, "order": order.to_dict()}

    def add_item(self, order_id: str, product_id: str, quantity: int) -> dict:
        """
        Add a product to an order. Validates product exists and has stock.

        Call chain: service -> order_repo.get -> product_repo.get -> product.is_available
                    -> product.reserve -> order.add_item -> order_repo.save
        """
        order = self._order_repo.get(order_id)
        if not order:
            return {"success": False, "error": "Order not found"}

        product = self._product_repo.get(product_id)
        if not product:
            return {"success": False, "error": "Product not found"}

        if not product.is_available(quantity):
            return {"success": False, "error": f"Insufficient stock for {product.name}"}

        product.reserve(quantity)
        item = OrderItem(
            product_id=product.id,
            product_name=product.name,
            quantity=quantity,
            unit_price=product.price,
        )
        order.add_item(item)
        self._order_repo.save(order)

        return {"success": True, "order": order.to_dict()}

    def confirm_order(self, order_id: str) -> dict:
        """Confirm an order (transition from draft to confirmed)."""
        order = self._order_repo.get(order_id)
        if not order:
            return {"success": False, "error": "Order not found"}

        try:
            order.confirm()
        except ValueError as e:
            return {"success": False, "error": str(e)}

        self._order_repo.save(order)
        return {"success": True, "order": order.to_dict()}

    def ship_order(self, order_id: str) -> dict:
        """Ship a confirmed order."""
        order = self._order_repo.get(order_id)
        if not order:
            return {"success": False, "error": "Order not found"}

        try:
            order.ship()
        except ValueError as e:
            return {"success": False, "error": str(e)}

        self._order_repo.save(order)
        return {"success": True, "order": order.to_dict()}

    def cancel_order(self, order_id: str) -> dict:
        """
        Cancel an order and release reserved stock.

        Call chain: service -> order_repo.get -> order.cancel
                    -> product_repo.get -> product.release -> order_repo.save
        """
        order = self._order_repo.get(order_id)
        if not order:
            return {"success": False, "error": "Order not found"}

        try:
            order.cancel()
        except ValueError as e:
            return {"success": False, "error": str(e)}

        # Release stock for all items
        for item in order.items:
            product = self._product_repo.get(item.product_id)
            if product:
                product.release(item.quantity)

        self._order_repo.save(order)
        return {"success": True, "order": order.to_dict()}

    def get_order(self, order_id: str) -> dict:
        """Get order details."""
        order = self._order_repo.get(order_id)
        if not order:
            return {"success": False, "error": "Order not found"}
        return {"success": True, "order": order.to_dict()}

    def list_customer_orders(self, customer_id: str) -> dict:
        """List all orders for a customer."""
        orders = self._order_repo.find_by_customer(customer_id)
        return {
            "success": True,
            "orders": [o.to_dict() for o in orders],
            "count": len(orders),
        }
