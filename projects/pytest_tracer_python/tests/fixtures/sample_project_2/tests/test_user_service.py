"""
Scenario tests for UserService.

These tests demonstrate:
- Class-based code coverage (User model, UserService)
- Shared utility coverage (validate_email called by multiple scenarios)
- Fixtures that execute code (sample_user)
"""

import pytest


@pytest.mark.scenario
@pytest.mark.behavior("user-management")
@pytest.mark.behavior("registration")
def test_create_user_success(user_service):
    """
    New user registers with valid email and name

    GIVEN valid user details
    WHEN creating a new user
    THEN user is created with generated ID
    """
    result = user_service.create_user(email="alice@example.com", name="Alice Smith")

    assert result["success"] is True
    assert result["user"]["email"] == "alice@example.com"
    assert result["user"]["name"] == "Alice Smith"
    assert result["user"]["id"].startswith("user-")
    assert result["user"]["is_active"] is True


@pytest.mark.scenario
@pytest.mark.behavior("user-management")
@pytest.mark.behavior("validation")
@pytest.mark.error
def test_create_user_invalid_email(user_service):
    """Registration fails with invalid email format"""
    result = user_service.create_user(email="not-an-email", name="Bob")

    assert result["success"] is False
    assert "email" in result["error"].lower()


@pytest.mark.scenario
@pytest.mark.behavior("user-management")
@pytest.mark.behavior("validation")
@pytest.mark.error
def test_create_user_empty_email(user_service):
    """Registration fails when email is empty"""
    result = user_service.create_user(email="", name="Bob")

    assert result["success"] is False
    assert "required" in result["error"].lower()


@pytest.mark.scenario
@pytest.mark.behavior("user-management")
@pytest.mark.behavior("validation")
@pytest.mark.error
def test_create_user_short_name(user_service):
    """Registration fails when name is too short"""
    result = user_service.create_user(email="bob@example.com", name="B")

    assert result["success"] is False
    assert "name" in result["error"].lower()


@pytest.mark.scenario
@pytest.mark.behavior("user-management")
@pytest.mark.error
def test_create_user_duplicate_email(user_service):
    """
    Registration fails when email already exists

    GIVEN an existing user with email
    WHEN another user tries to register with same email
    THEN registration is rejected
    """
    # First user
    user_service.create_user(email="taken@example.com", name="First")

    # Second user with same email
    result = user_service.create_user(email="taken@example.com", name="Second")

    assert result["success"] is False
    assert "already registered" in result["error"].lower()


@pytest.mark.scenario
@pytest.mark.behavior("user-management")
def test_get_user_success(user_service, sample_user):
    """Retrieve existing user by ID"""
    result = user_service.get_user(sample_user["id"])

    assert result["success"] is True
    assert result["user"]["id"] == sample_user["id"]


@pytest.mark.scenario
@pytest.mark.behavior("user-management")
@pytest.mark.error
def test_get_user_not_found(user_service):
    """Retrieving non-existent user returns error"""
    result = user_service.get_user("user-999")

    assert result["success"] is False
    assert "not found" in result["error"].lower()


@pytest.mark.scenario
@pytest.mark.behavior("user-management")
@pytest.mark.behavior("validation")
@pytest.mark.error
def test_get_user_invalid_id(user_service):
    """Retrieving user with invalid ID format returns error"""
    result = user_service.get_user("invalid-id")

    assert result["success"] is False
    assert "invalid" in result["error"].lower()


@pytest.mark.scenario
@pytest.mark.behavior("user-management")
@pytest.mark.behavior("account-status")
def test_deactivate_user(user_service, sample_user):
    """
    Admin deactivates a user account

    GIVEN an active user
    WHEN deactivating the user
    THEN user is marked inactive
    """
    result = user_service.deactivate_user(sample_user["id"])

    assert result["success"] is True
    assert result["user"]["is_active"] is False


@pytest.mark.scenario
@pytest.mark.behavior("user-management")
@pytest.mark.behavior("account-status")
@pytest.mark.error
def test_deactivate_already_inactive_user(user_service, sample_user):
    """Cannot deactivate an already inactive user"""
    user_service.deactivate_user(sample_user["id"])
    result = user_service.deactivate_user(sample_user["id"])

    assert result["success"] is False
    assert "already inactive" in result["error"].lower()


@pytest.mark.scenario
@pytest.mark.behavior("user-management")
@pytest.mark.behavior("listing")
def test_list_users(user_service):
    """List all users in the system"""
    user_service.create_user(email="a@example.com", name="Alice")
    user_service.create_user(email="b@example.com", name="Bob")

    result = user_service.list_users()

    assert result["success"] is True
    assert result["count"] == 2


@pytest.mark.scenario
@pytest.mark.behavior("user-management")
@pytest.mark.behavior("listing")
def test_list_active_users_only(user_service):
    """
    List only active users

    GIVEN multiple users with different statuses
    WHEN listing active users only
    THEN only active users are returned
    """
    user_service.create_user(email="active@example.com", name="Active")
    result = user_service.create_user(email="inactive@example.com", name="Inactive")
    user_service.deactivate_user(result["user"]["id"])

    result = user_service.list_users(active_only=True)

    assert result["success"] is True
    assert result["count"] == 1
    assert result["users"][0]["email"] == "active@example.com"
