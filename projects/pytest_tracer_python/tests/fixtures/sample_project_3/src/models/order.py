"""Order model with line items and status tracking."""

from enum import Enum


class OrderStatus(Enum):
    DRAFT = "draft"
    CONFIRMED = "confirmed"
    SHIPPED = "shipped"
    CANCELLED = "cancelled"


class OrderItem:
    def __init__(self, product_id: str, product_name: str, quantity: int, unit_price: float) -> None:
        self.product_id = product_id
        self.product_name = product_name
        self.quantity = quantity
        self.unit_price = unit_price

    @property
    def total(self) -> float:
        return self.quantity * self.unit_price

    def to_dict(self) -> dict:
        return {
            "product_id": self.product_id,
            "product_name": self.product_name,
            "quantity": self.quantity,
            "unit_price": self.unit_price,
            "total": self.total,
        }


class Order:
    def __init__(self, id: str, customer_id: str) -> None:
        self.id = id
        self.customer_id = customer_id
        self.items: list[OrderItem] = []
        self.status = OrderStatus.DRAFT

    def add_item(self, item: OrderItem) -> None:
        self.items.append(item)

    @property
    def total(self) -> float:
        return sum(item.total for item in self.items)

    def confirm(self) -> None:
        if self.status != OrderStatus.DRAFT:
            raise ValueError(f"Cannot confirm order in {self.status.value} status")
        if not self.items:
            raise ValueError("Cannot confirm order with no items")
        self.status = OrderStatus.CONFIRMED

    def ship(self) -> None:
        if self.status != OrderStatus.CONFIRMED:
            raise ValueError(f"Cannot ship order in {self.status.value} status")
        self.status = OrderStatus.SHIPPED

    def cancel(self) -> None:
        if self.status in (OrderStatus.SHIPPED,):
            raise ValueError("Cannot cancel shipped order")
        self.status = OrderStatus.CANCELLED

    def to_dict(self) -> dict:
        return {
            "id": self.id,
            "customer_id": self.customer_id,
            "items": [item.to_dict() for item in self.items],
            "total": self.total,
            "status": self.status.value,
        }
