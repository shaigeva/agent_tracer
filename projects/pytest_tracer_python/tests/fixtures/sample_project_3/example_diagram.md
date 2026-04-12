# Example Diagram: test_add_item_to_order

This scenario tests adding an item to an order through all 5 layers:
**route -> middleware -> service -> repository -> model**

## Scenario Diagram

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

# 3. Build trace index
trace build --coverage .coverage --scenarios scenarios.json --output .trace-index

# 4. Generate diagram
trace diagram "tests/test_order_flow.py::test_add_item_to_order" --index .trace-index

# 5. Extract mermaid to a viewable file
trace diagram "tests/test_order_flow.py::test_add_item_to_order" --index .trace-index \
  | python3 -c "
import sys, json
m = json.load(sys.stdin)['mermaid']
print('```mermaid')
print(m)
print('```')
" > diagram.md
```

## Viewing the diagram

- **GitHub**: Renders mermaid blocks natively in `.md` files
- **VS Code**: Install [Markdown Preview Mermaid Support](https://marketplace.visualstudio.com/items?itemName=bierner.markdown-mermaid) (`bierner.markdown-mermaid`), then Cmd+Shift+V to preview
- **Web**: Paste mermaid source at https://mermaid.live
