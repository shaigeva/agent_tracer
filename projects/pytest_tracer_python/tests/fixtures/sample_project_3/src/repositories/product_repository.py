"""In-memory product repository."""

from ..models import Product


class ProductRepository:
    def __init__(self) -> None:
        self._products: dict[str, Product] = {}

    def add(self, product: Product) -> None:
        self._products[product.id] = product

    def get(self, product_id: str) -> Product | None:
        return self._products.get(product_id)

    def list_all(self) -> list[Product]:
        return list(self._products.values())

    def update(self, product: Product) -> None:
        self._products[product.id] = product
