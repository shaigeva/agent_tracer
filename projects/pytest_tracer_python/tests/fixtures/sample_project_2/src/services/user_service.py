"""
User service demonstrating service layer with validation.

This service uses shared utilities (validation, formatting) to test
coverage of shared code paths across different scenarios.
"""

from ..models import User
from ..utils import validate_email, validate_id


class UserService:
    """Service for user operations."""

    def __init__(self):
        # In-memory storage for simplicity
        self._users: dict[str, User] = {}
        self._next_id = 1

    def create_user(self, email: str, name: str) -> dict:
        """
        Create a new user.

        Args:
            email: User's email address
            name: User's display name

        Returns:
            dict with user data or error
        """
        # Validate email using shared utility
        is_valid, error = validate_email(email)
        if not is_valid:
            return {"success": False, "error": error}

        if not name or len(name.strip()) < 2:
            return {"success": False, "error": "Name must be at least 2 characters"}

        # Check for duplicate email
        for user in self._users.values():
            if user.email == email:
                return {"success": False, "error": "Email already registered"}

        # Create user
        user_id = f"user-{self._next_id}"
        self._next_id += 1

        user = User(id=user_id, email=email, name=name.strip())
        self._users[user_id] = user

        return {"success": True, "user": user.to_dict()}

    def get_user(self, user_id: str) -> dict:
        """Get user by ID."""
        is_valid, error = validate_id(user_id, "user-")
        if not is_valid:
            return {"success": False, "error": error}

        user = self._users.get(user_id)
        if not user:
            return {"success": False, "error": "User not found"}

        return {"success": True, "user": user.to_dict()}

    def deactivate_user(self, user_id: str) -> dict:
        """Deactivate a user account."""
        is_valid, error = validate_id(user_id, "user-")
        if not is_valid:
            return {"success": False, "error": error}

        user = self._users.get(user_id)
        if not user:
            return {"success": False, "error": "User not found"}

        if not user.is_active:
            return {"success": False, "error": "User is already inactive"}

        user.deactivate()
        return {"success": True, "user": user.to_dict()}

    def list_users(self, active_only: bool = False) -> dict:
        """List all users."""
        users = list(self._users.values())
        if active_only:
            users = [u for u in users if u.is_active]

        return {
            "success": True,
            "users": [u.to_dict() for u in users],
            "count": len(users),
        }
