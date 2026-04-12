# Example Diagrams: test_add_item_to_order

This scenario tests adding an item to an order through all 5 layers:
**route -> middleware -> service -> repository -> model**

## Call-Chain Sequence Diagram (from sys.monitoring traces)

This shows the actual function calls in order, with arrows between files:

```mermaid
sequenceDiagram
    participant test as test_add_item_to_order
    participant routes as routes/order_routes.py
    participant middleware as middleware/auth.py
    participant services as services/order_service.py
    participant repos as repositories/order_repository.py
    participant products as repositories/product_repository.py
    participant order_model as models/order.py
    participant product_model as models/product.py
    test ->> routes: OrderRoutes.post_order
    routes ->> middleware: AuthMiddleware.create_order
    middleware ->> services: OrderService.create_order
    services ->> repos: OrderRepository.next_id
    services ->> order_model: Order.__init__
    services ->> repos: OrderRepository.save
    test ->> routes: OrderRoutes.post_order_item
    routes ->> middleware: AuthMiddleware.add_item
    middleware ->> services: OrderService.add_item
    services ->> repos: OrderRepository.get
    services ->> products: ProductRepository.get
    services ->> product_model: Product.is_available
    services ->> product_model: Product.reserve
    services ->> order_model: OrderItem.__init__
    services ->> order_model: Order.add_item
    services ->> repos: OrderRepository.save
```

## Coverage Diagram (from pytest-cov line coverage)

This shows which files are touched, grouped by directory:

```mermaid
graph TD
    test["test_add_item_to_order"]
    subgraph routes["routes"]
        routes_order_routes_py["order_routes.py<br/>(2 lines)"]
    end
    subgraph middleware["middleware"]
        middleware_auth_py["auth.py<br/>(8 lines)"]
    end
    subgraph services["services"]
        services_order_service_py["order_service.py<br/>(18 lines)"]
    end
    subgraph repositories["repositories"]
        repositories_order_repository_py["order_repository.py<br/>(5 lines)"]
        repositories_product_repository_py["product_repository.py<br/>(1 lines)"]
    end
    subgraph models["models"]
        models_order_py["order.py<br/>(23 lines)"]
        models_product_py["product.py<br/>(3 lines)"]
    end
    test --> routes_order_routes_py
    test --> middleware_auth_py
    test --> services_order_service_py
    test --> repositories_order_repository_py
    test --> repositories_product_repository_py
    test --> models_order_py
    test --> models_product_py
```

## How this was generated

```bash
# 1. Run tests with coverage
uv run pytest tests/ --cov=src --cov-context=test

# 2. Collect scenario metadata
uv run pytest-tracer collect . -o scenarios.json

# 3. Collect call traces (uses sys.monitoring)
uv run pytest-tracer trace . -o call_traces.json

# 4. Build trace index with call traces
trace build --coverage .coverage --scenarios scenarios.json \
  --call-traces call_traces.json --output .trace-index

# 5. Generate sequence diagram (call chain)
trace flamegraph "tests/test_order_flow.py::test_add_item_to_order" \
  --format mermaid --index .trace-index

# 6. Generate folded stacks (for speedscope flame graph viewer)
trace flamegraph "tests/test_order_flow.py::test_add_item_to_order" \
  --index .trace-index > profile.folded

# 7. Generate coverage diagram
trace diagram "tests/test_order_flow.py::test_add_item_to_order" --index .trace-index
```

## Viewing

- **Sequence diagrams**: GitHub renders mermaid natively. VS Code needs the "Markdown Preview Mermaid Support" extension (`bierner.markdown-mermaid`)
- **Flame graphs**: Load the folded stacks file in [speedscope](https://www.speedscope.app/) or pipe through `flamegraph.pl`
