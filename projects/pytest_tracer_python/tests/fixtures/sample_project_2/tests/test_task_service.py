"""
Scenario tests for TaskService.

These tests demonstrate:
- Cross-service dependencies (TaskService -> UserService)
- State machine behavior (task lifecycle)
- Parametrized tests with @scenario marker
"""

import pytest
from src.models import TaskStatus


@pytest.mark.scenario
@pytest.mark.behavior("task-management")
def test_create_task_unassigned(task_service):
    """Create a task without assignee"""
    result = task_service.create_task(title="New Task")

    assert result["success"] is True
    assert result["task"]["title"] == "New Task"
    assert result["task"]["assignee_id"] is None
    assert result["task"]["status"] == "pending"


@pytest.mark.scenario
@pytest.mark.behavior("task-management")
@pytest.mark.behavior("assignment")
def test_create_task_with_assignee(task_service, sample_user):
    """
    Create a task assigned to a user

    GIVEN an existing user
    WHEN creating a task with that user as assignee
    THEN task is created with the assignment
    """
    result = task_service.create_task(title="Assigned Task", assignee_id=sample_user["id"])

    assert result["success"] is True
    assert result["task"]["assignee_id"] == sample_user["id"]


@pytest.mark.scenario
@pytest.mark.behavior("task-management")
@pytest.mark.behavior("validation")
@pytest.mark.error
def test_create_task_invalid_assignee(task_service):
    """Task creation fails with non-existent assignee"""
    result = task_service.create_task(title="Task", assignee_id="user-nonexistent")

    assert result["success"] is False
    assert "assignee" in result["error"].lower()


@pytest.mark.scenario
@pytest.mark.behavior("task-management")
@pytest.mark.behavior("validation")
@pytest.mark.error
def test_create_task_short_title(task_service):
    """Task creation fails with title too short"""
    result = task_service.create_task(title="AB")

    assert result["success"] is False
    assert "title" in result["error"].lower()


@pytest.mark.scenario
@pytest.mark.behavior("task-management")
def test_get_task(task_service, sample_task):
    """Retrieve task by ID"""
    result = task_service.get_task(sample_task["id"])

    assert result["success"] is True
    assert result["task"]["id"] == sample_task["id"]


@pytest.mark.scenario
@pytest.mark.behavior("task-management")
@pytest.mark.error
def test_get_task_not_found(task_service):
    """Retrieving non-existent task returns error"""
    result = task_service.get_task("task-999")

    assert result["success"] is False
    assert "not found" in result["error"].lower()


@pytest.mark.scenario
@pytest.mark.behavior("task-management")
@pytest.mark.behavior("assignment")
def test_assign_task(task_service, sample_user):
    """
    Assign existing task to a user

    GIVEN an unassigned task and a user
    WHEN assigning the task to the user
    THEN task shows the new assignment
    """
    create_result = task_service.create_task(title="Unassigned Task")
    task_id = create_result["task"]["id"]

    result = task_service.assign_task(task_id, sample_user["id"])

    assert result["success"] is True
    assert result["task"]["assignee_id"] == sample_user["id"]


@pytest.mark.scenario
@pytest.mark.behavior("task-management")
@pytest.mark.behavior("assignment")
@pytest.mark.error
def test_assign_task_invalid_user(task_service):
    """Cannot assign task to non-existent user"""
    create_result = task_service.create_task(title="Task")
    task_id = create_result["task"]["id"]

    result = task_service.assign_task(task_id, "user-nonexistent")

    assert result["success"] is False
    assert "assignee" in result["error"].lower()


# Task lifecycle tests


@pytest.mark.scenario
@pytest.mark.behavior("task-management")
@pytest.mark.behavior("lifecycle")
def test_start_task(task_service):
    """
    Start working on a pending task

    GIVEN a task in pending status
    WHEN starting the task
    THEN task status changes to in_progress
    """
    create_result = task_service.create_task(title="Task to Start")
    task_id = create_result["task"]["id"]

    result = task_service.start_task(task_id)

    assert result["success"] is True
    assert result["task"]["status"] == "in_progress"


@pytest.mark.scenario
@pytest.mark.behavior("task-management")
@pytest.mark.behavior("lifecycle")
@pytest.mark.error
def test_start_task_already_started(task_service):
    """Cannot start a task that's already in progress"""
    create_result = task_service.create_task(title="Task")
    task_id = create_result["task"]["id"]
    task_service.start_task(task_id)

    result = task_service.start_task(task_id)

    assert result["success"] is False
    assert "cannot start" in result["error"].lower()


@pytest.mark.scenario
@pytest.mark.behavior("task-management")
@pytest.mark.behavior("lifecycle")
def test_complete_task(task_service):
    """
    Complete a task

    GIVEN a task in progress
    WHEN completing the task
    THEN task status changes to completed
    """
    create_result = task_service.create_task(title="Task to Complete")
    task_id = create_result["task"]["id"]
    task_service.start_task(task_id)

    result = task_service.complete_task(task_id)

    assert result["success"] is True
    assert result["task"]["status"] == "completed"


@pytest.mark.scenario
@pytest.mark.behavior("task-management")
@pytest.mark.behavior("lifecycle")
def test_complete_pending_task(task_service):
    """Can complete a pending task directly (skip in_progress)"""
    create_result = task_service.create_task(title="Task")
    task_id = create_result["task"]["id"]

    result = task_service.complete_task(task_id)

    assert result["success"] is True
    assert result["task"]["status"] == "completed"


@pytest.mark.scenario
@pytest.mark.behavior("task-management")
@pytest.mark.behavior("lifecycle")
def test_cancel_task(task_service):
    """Cancel a pending task"""
    create_result = task_service.create_task(title="Task to Cancel")
    task_id = create_result["task"]["id"]

    result = task_service.cancel_task(task_id)

    assert result["success"] is True
    assert result["task"]["status"] == "cancelled"


@pytest.mark.scenario
@pytest.mark.behavior("task-management")
@pytest.mark.behavior("lifecycle")
@pytest.mark.error
def test_cancel_completed_task(task_service):
    """Cannot cancel a completed task"""
    create_result = task_service.create_task(title="Task")
    task_id = create_result["task"]["id"]
    task_service.complete_task(task_id)

    result = task_service.cancel_task(task_id)

    assert result["success"] is False
    assert "cannot cancel" in result["error"].lower()


@pytest.mark.scenario
@pytest.mark.behavior("task-management")
@pytest.mark.behavior("lifecycle")
@pytest.mark.error
def test_complete_cancelled_task(task_service):
    """Cannot complete a cancelled task"""
    create_result = task_service.create_task(title="Task")
    task_id = create_result["task"]["id"]
    task_service.cancel_task(task_id)

    result = task_service.complete_task(task_id)

    assert result["success"] is False
    assert "cannot complete" in result["error"].lower()


@pytest.mark.scenario
@pytest.mark.behavior("task-management")
@pytest.mark.behavior("listing")
def test_list_tasks(task_service):
    """List all tasks"""
    task_service.create_task(title="Task 1")
    task_service.create_task(title="Task 2")

    result = task_service.list_tasks()

    assert result["success"] is True
    assert result["count"] == 2


@pytest.mark.scenario
@pytest.mark.behavior("task-management")
@pytest.mark.behavior("listing")
@pytest.mark.behavior("filtering")
def test_list_tasks_by_status(task_service):
    """
    Filter tasks by status

    GIVEN tasks in different statuses
    WHEN filtering by a specific status
    THEN only matching tasks are returned
    """
    result1 = task_service.create_task(title="Task 1")
    task_service.create_task(title="Task 2")  # Create second task (unused result)
    task_service.start_task(result1["task"]["id"])

    result = task_service.list_tasks(status=TaskStatus.IN_PROGRESS)

    assert result["success"] is True
    assert result["count"] == 1
    assert result["tasks"][0]["title"] == "Task 1"


@pytest.mark.scenario
@pytest.mark.behavior("task-management")
@pytest.mark.behavior("listing")
@pytest.mark.behavior("filtering")
def test_list_tasks_by_assignee(task_service, sample_user):
    """Filter tasks by assignee"""
    task_service.create_task(title="Unassigned")
    task_service.create_task(title="Assigned", assignee_id=sample_user["id"])

    result = task_service.list_tasks(assignee_id=sample_user["id"])

    assert result["success"] is True
    assert result["count"] == 1
    assert result["tasks"][0]["title"] == "Assigned"
