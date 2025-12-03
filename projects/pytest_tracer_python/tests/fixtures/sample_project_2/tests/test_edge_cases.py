"""
Edge case tests demonstrating various pytest-tracer scenarios.

These tests demonstrate:
- Empty/missing docstrings
- Parametrized tests with @scenario
- Tests with no behavior markers
"""

import pytest


@pytest.mark.scenario
def test_scenario_without_behavior(user_service):
    """A scenario test without any behavior markers"""
    result = user_service.list_users()
    assert result["success"] is True


@pytest.mark.scenario
@pytest.mark.behavior("edge-cases")
def test_scenario_with_empty_docstring(user_service):
    result = user_service.list_users()
    assert result["success"] is True


@pytest.mark.scenario
@pytest.mark.behavior("validation")
@pytest.mark.behavior("email")
@pytest.mark.parametrize(
    "email,should_pass",
    [
        ("valid@example.com", True),
        ("also.valid@test.org", True),
        ("no-at-sign", False),
        ("@no-local-part.com", False),
        ("no-domain@", False),
        ("", False),
    ],
)
def test_email_validation_variants(user_service, email, should_pass):
    """
    Email validation handles various formats

    GIVEN different email formats
    WHEN attempting to create a user
    THEN validation correctly accepts or rejects
    """
    result = user_service.create_user(email=email, name="Test User")

    if should_pass:
        assert result["success"] is True
    else:
        assert result["success"] is False


@pytest.mark.scenario
@pytest.mark.behavior("task-management")
@pytest.mark.behavior("validation")
@pytest.mark.parametrize(
    "title",
    [
        "A",
        "AB",
        "",
        "   ",
    ],
)
@pytest.mark.error
def test_task_title_too_short(task_service, title):
    """Task titles must be at least 3 characters"""
    result = task_service.create_task(title=title)
    assert result["success"] is False


# A regular test (not a scenario) - should NOT be collected
def test_non_scenario_helper():
    """This test does not have @scenario marker and should not appear in scenarios.json"""
    assert True


# Class-based test demonstrating class test collection
class TestUserServiceClass:
    """Class-based tests - these should also be collected if marked."""

    @pytest.mark.scenario
    @pytest.mark.behavior("user-management")
    @pytest.mark.behavior("class-based-test")
    def test_class_based_scenario(self, user_service):
        """
        Class-based scenario test

        GIVEN a class-based test structure
        WHEN the test has @scenario marker
        THEN it should be collected like function tests
        """
        result = user_service.create_user(email="class-test@example.com", name="Class Test")
        assert result["success"] is True
