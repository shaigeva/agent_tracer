"""
Pydantic models for scenario metadata.

These models define the structure of scenario test metadata that is:
1. Extracted from pytest test collection
2. Serialized to JSON for the Rust analyzer
3. Used throughout the pytest-tracer system
"""

from datetime import datetime
from typing import Literal

from pydantic import BaseModel, Field


class ScenarioMetadata(BaseModel):
    """
    Metadata for a single scenario test.

    A scenario test is a pytest test decorated with @pytest.mark.scenario
    that serves as a documented entry point for understanding code behavior.
    """

    id: str = Field(description="Full pytest node ID (e.g., 'tests/test_auth.py::test_login')")
    file: str = Field(description="Path to the test file relative to project root")
    function: str = Field(description="Name of the test function")
    description: str = Field(description="Short description (first line of docstring)")
    documentation: str | None = Field(default=None, description="Full docstring including GIVEN/WHEN/THEN sections")
    behaviors: list[str] = Field(
        default_factory=list,
        description="List of behavior tags from @pytest.mark.behavior markers",
    )
    outcome: Literal["success", "error"] = Field(
        default="success",
        description="Expected outcome: 'error' if @pytest.mark.error, else 'success'",
    )


class ScenariosFile(BaseModel):
    """
    Root model for the scenarios JSON file.

    This is the format consumed by the Rust trace-analyzer.
    """

    version: str = Field(default="1.0", description="Schema version")
    collected_at: datetime = Field(
        default_factory=datetime.now,
        description="Timestamp when scenarios were collected",
    )
    scenarios: list[ScenarioMetadata] = Field(default_factory=list, description="List of collected scenarios")

    def to_json(self, indent: int = 2) -> str:
        """Serialize to JSON string."""
        return self.model_dump_json(indent=indent)

    @classmethod
    def from_json(cls, json_str: str) -> "ScenariosFile":
        """Deserialize from JSON string."""
        return cls.model_validate_json(json_str)
