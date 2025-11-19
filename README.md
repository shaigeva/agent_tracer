# Python repo template

# Setup

## Python version:
If you don't want 3.14, change:
- .python-version
- pyproject.toml (there are a couple of places)

Install (also create virtualenv if needed):
```sh
uv sync
```

# Run
Run main:
```sh
uv run python -m proj_name.main
```

# Running Tests

```sh
# All validations (lint, format, type check, tests)
./devtools/run_all_agent_validations.sh

# Just tests
uv run pytest -rP

# Watch mode
./devtools/run_tests_watch.sh
```
