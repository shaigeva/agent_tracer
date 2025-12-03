"""
Task service demonstrating cross-service dependencies.

This service depends on UserService for assignee validation,
testing cross-module coverage tracking.
"""

from ..models import Task, TaskStatus
from ..utils import validate_id
from .user_service import UserService


class TaskService:
    """Service for task operations."""

    def __init__(self, user_service: UserService):
        self._user_service = user_service
        self._tasks: dict[str, Task] = {}
        self._next_id = 1

    def create_task(self, title: str, assignee_id: str | None = None) -> dict:
        """
        Create a new task.

        Args:
            title: Task title
            assignee_id: Optional user ID to assign the task to

        Returns:
            dict with task data or error
        """
        if not title or len(title.strip()) < 3:
            return {"success": False, "error": "Title must be at least 3 characters"}

        # Validate assignee if provided
        if assignee_id:
            user_result = self._user_service.get_user(assignee_id)
            if not user_result["success"]:
                return {"success": False, "error": f"Invalid assignee: {user_result['error']}"}

        task_id = f"task-{self._next_id}"
        self._next_id += 1

        task = Task(id=task_id, title=title.strip(), assignee_id=assignee_id)
        self._tasks[task_id] = task

        return {"success": True, "task": task.to_dict()}

    def get_task(self, task_id: str) -> dict:
        """Get task by ID."""
        is_valid, error = validate_id(task_id, "task-")
        if not is_valid:
            return {"success": False, "error": error}

        task = self._tasks.get(task_id)
        if not task:
            return {"success": False, "error": "Task not found"}

        return {"success": True, "task": task.to_dict()}

    def assign_task(self, task_id: str, user_id: str) -> dict:
        """Assign a task to a user."""
        # Validate task
        is_valid, error = validate_id(task_id, "task-")
        if not is_valid:
            return {"success": False, "error": error}

        task = self._tasks.get(task_id)
        if not task:
            return {"success": False, "error": "Task not found"}

        # Validate user exists
        user_result = self._user_service.get_user(user_id)
        if not user_result["success"]:
            return {"success": False, "error": f"Invalid assignee: {user_result['error']}"}

        task.assign(user_id)
        return {"success": True, "task": task.to_dict()}

    def start_task(self, task_id: str) -> dict:
        """Start a task."""
        is_valid, error = validate_id(task_id, "task-")
        if not is_valid:
            return {"success": False, "error": error}

        task = self._tasks.get(task_id)
        if not task:
            return {"success": False, "error": "Task not found"}

        try:
            task.start()
        except ValueError as e:
            return {"success": False, "error": str(e)}

        return {"success": True, "task": task.to_dict()}

    def complete_task(self, task_id: str) -> dict:
        """Complete a task."""
        is_valid, error = validate_id(task_id, "task-")
        if not is_valid:
            return {"success": False, "error": error}

        task = self._tasks.get(task_id)
        if not task:
            return {"success": False, "error": "Task not found"}

        try:
            task.complete()
        except ValueError as e:
            return {"success": False, "error": str(e)}

        return {"success": True, "task": task.to_dict()}

    def cancel_task(self, task_id: str) -> dict:
        """Cancel a task."""
        is_valid, error = validate_id(task_id, "task-")
        if not is_valid:
            return {"success": False, "error": error}

        task = self._tasks.get(task_id)
        if not task:
            return {"success": False, "error": "Task not found"}

        try:
            task.cancel()
        except ValueError as e:
            return {"success": False, "error": str(e)}

        return {"success": True, "task": task.to_dict()}

    def list_tasks(self, status: TaskStatus | None = None, assignee_id: str | None = None) -> dict:
        """List tasks with optional filters."""
        tasks = list(self._tasks.values())

        if status:
            tasks = [t for t in tasks if t.status == status]

        if assignee_id:
            tasks = [t for t in tasks if t.assignee_id == assignee_id]

        return {
            "success": True,
            "tasks": [t.to_dict() for t in tasks],
            "count": len(tasks),
        }
