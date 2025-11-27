"""
Sample authentication module for testing pytest-tracer.

This module provides simple authentication functions that are exercised
by the scenario tests in tests/test_auth.py.
"""


def login(email: str, password: str) -> dict:
    """
    Authenticate a user with email and password.

    Args:
        email: User's email address
        password: User's password

    Returns:
        dict with status code and token/error
    """
    # Simple validation
    if not email or "@" not in email:
        return {"status": 400, "error": "Invalid email format"}

    if not password:
        return {"status": 400, "error": "Password required"}

    # Simulate authentication
    if password == "valid_password":
        return {"status": 200, "token": "abc123token"}

    return {"status": 401, "error": "Invalid credentials"}


def logout(token: str) -> dict:
    """
    Invalidate a user session.

    Args:
        token: Session token to invalidate

    Returns:
        dict with status code
    """
    if not token:
        return {"status": 400, "error": "Token required"}

    # Simulate logout
    return {"status": 200, "message": "Logged out successfully"}


def get_user_profile(token: str) -> dict:
    """
    Get user profile for authenticated user.

    Args:
        token: Valid session token

    Returns:
        dict with user profile or error
    """
    if token == "abc123token":
        return {
            "status": 200,
            "user": {"id": "user-1", "email": "user@example.com", "name": "Test User"},
        }

    return {"status": 401, "error": "Invalid token"}
