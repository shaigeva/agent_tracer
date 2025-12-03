"""
Pytest configuration and fixtures for sample_project_2.

This file demonstrates fixtures that execute code, which should
be tracked in coverage when scenarios use them.
"""

import pytest
from src.services import TaskService, UserService


@pytest.fixture
def user_service():
    """Provide a fresh UserService instance."""
    return UserService()


@pytest.fixture
def task_service(user_service):
    """Provide a TaskService with its UserService dependency."""
    return TaskService(user_service)


@pytest.fixture
def sample_user(user_service):
    """Create a sample user for tests that need an existing user."""
    result = user_service.create_user(email="test@example.com", name="Test User")
    return result["user"]


@pytest.fixture
def sample_task(task_service, sample_user):
    """Create a sample task assigned to the sample user."""
    result = task_service.create_task(title="Sample Task", assignee_id=sample_user["id"])
    return result["task"]
