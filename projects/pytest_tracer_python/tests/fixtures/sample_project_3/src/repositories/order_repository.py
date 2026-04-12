"""In-memory order repository."""

from ..models import Order, OrderStatus


class OrderRepository:
    def __init__(self) -> None:
        self._orders: dict[str, Order] = {}
        self._next_id = 1

    def next_id(self) -> str:
        order_id = f"order-{self._next_id}"
        self._next_id += 1
        return order_id

    def save(self, order: Order) -> None:
        self._orders[order.id] = order

    def get(self, order_id: str) -> Order | None:
        return self._orders.get(order_id)

    def find_by_customer(self, customer_id: str) -> list[Order]:
        return [o for o in self._orders.values() if o.customer_id == customer_id]

    def find_by_status(self, status: OrderStatus) -> list[Order]:
        return [o for o in self._orders.values() if o.status == status]
