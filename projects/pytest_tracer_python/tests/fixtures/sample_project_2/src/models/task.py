"""
Task model demonstrating enums and complex state.
"""

from enum import Enum

from .base import BaseModel


class TaskStatus(Enum):
    """Task status enumeration."""

    PENDING = "pending"
    IN_PROGRESS = "in_progress"
    COMPLETED = "completed"
    CANCELLED = "cancelled"


class Task(BaseModel):
    """Task domain model."""

    def __init__(self, id: str, title: str, assignee_id: str | None = None):
        super().__init__(id)
        self.title = title
        self.assignee_id = assignee_id
        self.status = TaskStatus.PENDING
        self.description = ""

    def assign(self, user_id: str):
        """Assign task to a user."""
        self.assignee_id = user_id
        self.touch()

    def start(self):
        """Start working on the task."""
        if self.status != TaskStatus.PENDING:
            raise ValueError(f"Cannot start task in {self.status.value} status")
        self.status = TaskStatus.IN_PROGRESS
        self.touch()

    def complete(self):
        """Mark task as completed."""
        if self.status == TaskStatus.CANCELLED:
            raise ValueError("Cannot complete a cancelled task")
        self.status = TaskStatus.COMPLETED
        self.touch()

    def cancel(self):
        """Cancel the task."""
        if self.status == TaskStatus.COMPLETED:
            raise ValueError("Cannot cancel a completed task")
        self.status = TaskStatus.CANCELLED
        self.touch()

    def to_dict(self) -> dict:
        """Convert task to dictionary."""
        data = super().to_dict()
        data.update(
            {
                "title": self.title,
                "assignee_id": self.assignee_id,
                "status": self.status.value,
                "description": self.description,
            }
        )
        return data
