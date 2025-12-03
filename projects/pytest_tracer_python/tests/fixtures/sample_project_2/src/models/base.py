"""
Base model class demonstrating class-based code coverage.
"""

from datetime import datetime


class BaseModel:
    """Base class for all domain models."""

    def __init__(self, id: str):
        self.id = id
        self.created_at = datetime.now()
        self.updated_at = datetime.now()

    def touch(self):
        """Update the updated_at timestamp."""
        self.updated_at = datetime.now()

    def to_dict(self) -> dict:
        """Convert model to dictionary."""
        return {
            "id": self.id,
            "created_at": self.created_at.isoformat(),
            "updated_at": self.updated_at.isoformat(),
        }
