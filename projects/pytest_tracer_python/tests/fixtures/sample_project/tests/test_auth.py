"""
Scenario tests for authentication functionality.

These tests demonstrate the pytest-tracer markers and are used
to test the scenario collector.
"""

import pytest
from src.auth import get_user_profile, login, logout


@pytest.mark.scenario
@pytest.mark.behavior("authentication")
@pytest.mark.behavior("session-management")
def test_successful_login():
    """
    User logs in with valid credentials

    GIVEN a registered user with email and password
    WHEN they submit valid credentials
    THEN they receive an auth token
    """
    result = login("user@example.com", "valid_password")
    assert result["status"] == 200
    assert result["token"] is not None


@pytest.mark.scenario
@pytest.mark.behavior("authentication")
@pytest.mark.error
def test_login_invalid_password():
    """Login fails with wrong password"""
    result = login("user@example.com", "wrong_password")
    assert result["status"] == 401
    assert "Invalid credentials" in result["error"]


@pytest.mark.scenario
@pytest.mark.behavior("authentication")
@pytest.mark.behavior("validation")
@pytest.mark.error
def test_login_invalid_email():
    """
    Login fails with invalid email format

    GIVEN an invalid email address
    WHEN user attempts to login
    THEN they receive a validation error
    """
    result = login("not-an-email", "any_password")
    assert result["status"] == 400
    assert "Invalid email" in result["error"]


@pytest.mark.scenario
@pytest.mark.behavior("session-management")
def test_logout():
    """User logs out and session is invalidated"""
    result = logout("abc123token")
    assert result["status"] == 200


@pytest.mark.scenario
@pytest.mark.behavior("authentication")
@pytest.mark.behavior("user-profile")
def test_get_profile_with_valid_token():
    """
    Authenticated user retrieves their profile

    GIVEN a logged-in user with valid token
    WHEN they request their profile
    THEN they receive their user information
    """
    result = get_user_profile("abc123token")
    assert result["status"] == 200
    assert result["user"]["email"] == "user@example.com"


@pytest.mark.scenario
@pytest.mark.behavior("authentication")
@pytest.mark.error
def test_get_profile_with_invalid_token():
    """Profile access denied with invalid token"""
    result = get_user_profile("invalid_token")
    assert result["status"] == 401


# This test is NOT a scenario - should not be collected
def test_regular_helper_function():
    """This is not a scenario test, just a regular test."""
    assert True
