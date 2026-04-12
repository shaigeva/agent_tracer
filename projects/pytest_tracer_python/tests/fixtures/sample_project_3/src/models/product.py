"""Product model with inventory tracking."""


class Product:
    def __init__(self, id: str, name: str, price: float, stock: int) -> None:
        self.id = id
        self.name = name
        self.price = price
        self.stock = stock

    def is_available(self, quantity: int = 1) -> bool:
        return self.stock >= quantity

    def reserve(self, quantity: int) -> None:
        if not self.is_available(quantity):
            raise ValueError(f"Insufficient stock for {self.name}: need {quantity}, have {self.stock}")
        self.stock -= quantity

    def release(self, quantity: int) -> None:
        self.stock += quantity

    def to_dict(self) -> dict:
        return {
            "id": self.id,
            "name": self.name,
            "price": self.price,
            "stock": self.stock,
        }
