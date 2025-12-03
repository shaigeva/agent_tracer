"""
Shared validation utilities.

These functions are called by multiple services, testing coverage
of shared code paths across different scenarios.
"""

import re


def validate_email(email: str) -> tuple[bool, str | None]:
    """
    Validate email format.

    Returns:
        Tuple of (is_valid, error_message)
    """
    if not email:
        return False, "Email is required"

    if not re.match(r"^[^@]+@[^@]+\.[^@]+$", email):
        return False, "Invalid email format"

    return True, None


def validate_id(id_value: str, prefix: str) -> tuple[bool, str | None]:
    """
    Validate ID format with expected prefix.

    Args:
        id_value: The ID to validate
        prefix: Expected prefix (e.g., 'user-', 'task-')

    Returns:
        Tuple of (is_valid, error_message)
    """
    if not id_value:
        return False, f"{prefix.rstrip('-').title()} ID is required"

    if not id_value.startswith(prefix):
        return False, f"Invalid {prefix.rstrip('-')} ID format"

    return True, None
