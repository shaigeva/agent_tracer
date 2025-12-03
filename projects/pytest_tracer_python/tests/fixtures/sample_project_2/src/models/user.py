"""
User model with inheritance from BaseModel.
"""

from .base import BaseModel


class User(BaseModel):
    """User domain model."""

    def __init__(self, id: str, email: str, name: str):
        super().__init__(id)
        self.email = email
        self.name = name
        self.is_active = True

    def deactivate(self):
        """Deactivate the user account."""
        self.is_active = False
        self.touch()

    def activate(self):
        """Activate the user account."""
        self.is_active = True
        self.touch()

    def to_dict(self) -> dict:
        """Convert user to dictionary."""
        data = super().to_dict()
        data.update(
            {
                "email": self.email,
                "name": self.name,
                "is_active": self.is_active,
            }
        )
        return data
