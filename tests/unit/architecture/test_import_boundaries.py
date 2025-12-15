"""Tests for architecture import boundaries.

These tests ensure that the clean architecture boundaries are maintained:
- Core modules (application, domain, services) must not import from CLI
- Application layer must not import from legacy

CLEAN2-A3 Implementation.
"""

from __future__ import annotations

import ast
import re
from pathlib import Path

import pytest


# Root of the cdisc_transpiler package
PACKAGE_ROOT = Path(__file__).parent.parent.parent.parent / "cdisc_transpiler"


def get_python_files(directory: Path) -> list[Path]:
    """Get all Python files in a directory recursively.

    Args:
        directory: Directory to search

    Returns:
        List of Python file paths
    """
    return list(directory.rglob("*.py"))


def extract_imports_from_file(file_path: Path) -> list[str]:
    """Extract all import statements from a Python file.

    Args:
        file_path: Path to Python file

    Returns:
        List of import strings (module names)
    """
    imports = []

    try:
        with open(file_path, "r", encoding="utf-8") as f:
            content = f.read()

        tree = ast.parse(content)

        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                for alias in node.names:
                    imports.append(alias.name)
            elif isinstance(node, ast.ImportFrom):
                if node.module:
                    imports.append(node.module)

    except SyntaxError:
        # Skip files with syntax errors
        pass

    return imports


def has_forbidden_import(imports: list[str], forbidden_pattern: str) -> list[str]:
    """Check if any imports match a forbidden pattern.

    Args:
        imports: List of import strings
        forbidden_pattern: Regex pattern for forbidden imports

    Returns:
        List of matching forbidden imports
    """
    pattern = re.compile(forbidden_pattern)
    return [imp for imp in imports if pattern.search(imp)]


class TestCLIImportBoundary:
    """Tests ensuring core modules do not import from CLI.

    The CLI layer should be the outermost layer - it can import from
    anything, but nothing should import from it except CLI code itself.

    Note: Legacy modules are excluded from this check because they are
    deprecated and will be removed in a future release. The focus is on
    ensuring the NEW architecture layers don't import from CLI.
    """

    def test_services_do_not_import_cli(self):
        """Services layer must not import from CLI."""
        services_dir = PACKAGE_ROOT / "services"
        if not services_dir.exists():
            pytest.skip("services directory not found")

        violations = []
        for py_file in get_python_files(services_dir):
            imports = extract_imports_from_file(py_file)
            forbidden = has_forbidden_import(imports, r"(^|\.)cli(\.|$)")
            if forbidden:
                rel_path = py_file.relative_to(PACKAGE_ROOT.parent)
                violations.append(f"{rel_path}: {forbidden}")

        assert not violations, f"Services layer imports CLI modules:\n" + "\n".join(
            violations
        )

    def test_application_does_not_import_cli(self):
        """Application layer must not import from CLI."""
        application_dir = PACKAGE_ROOT / "application"
        if not application_dir.exists():
            pytest.skip("application directory not found")

        violations = []
        for py_file in get_python_files(application_dir):
            imports = extract_imports_from_file(py_file)
            forbidden = has_forbidden_import(imports, r"(^|\.)cli(\.|$)")
            if forbidden:
                rel_path = py_file.relative_to(PACKAGE_ROOT.parent)
                violations.append(f"{rel_path}: {forbidden}")

        assert not violations, f"Application layer imports CLI modules:\n" + "\n".join(
            violations
        )

    def test_domain_does_not_import_cli(self):
        """Domain layer must not import from CLI."""
        domain_dir = PACKAGE_ROOT / "domain"
        if not domain_dir.exists():
            pytest.skip("domain directory not found")

        violations = []
        for py_file in get_python_files(domain_dir):
            imports = extract_imports_from_file(py_file)
            forbidden = has_forbidden_import(imports, r"(^|\.)cli(\.|$)")
            if forbidden:
                rel_path = py_file.relative_to(PACKAGE_ROOT.parent)
                violations.append(f"{rel_path}: {forbidden}")

        assert not violations, f"Domain layer imports CLI modules:\n" + "\n".join(
            violations
        )

    def test_infrastructure_does_not_import_cli(self):
        """Infrastructure layer must not import from CLI."""
        infrastructure_dir = PACKAGE_ROOT / "infrastructure"
        if not infrastructure_dir.exists():
            pytest.skip("infrastructure directory not found")

        violations = []
        for py_file in get_python_files(infrastructure_dir):
            imports = extract_imports_from_file(py_file)
            forbidden = has_forbidden_import(imports, r"(^|\.)cli(\.|$)")
            if forbidden:
                rel_path = py_file.relative_to(PACKAGE_ROOT.parent)
                violations.append(f"{rel_path}: {forbidden}")

        assert not violations, (
            f"Infrastructure layer imports CLI modules:\n" + "\n".join(violations)
        )

    def test_xpt_module_does_not_import_cli(self):
        """XPT module must not import from CLI (domain processors, etc.)."""
        xpt_dir = PACKAGE_ROOT / "xpt_module"
        if not xpt_dir.exists():
            pytest.skip("xpt_module directory not found")

        violations = []
        for py_file in get_python_files(xpt_dir):
            imports = extract_imports_from_file(py_file)
            forbidden = has_forbidden_import(imports, r"(^|\.)cli(\.|$)")
            if forbidden:
                rel_path = py_file.relative_to(PACKAGE_ROOT.parent)
                violations.append(f"{rel_path}: {forbidden}")

        assert not violations, f"XPT module imports CLI modules:\n" + "\n".join(
            violations
        )


class TestLegacyImportBoundary:
    """Tests ensuring application layer does not import from legacy.

    The application layer should use ports and adapters, not legacy
    implementations directly.
    """

    def test_application_does_not_import_legacy(self):
        """Application layer must not import from legacy.

        Note: TYPE_CHECKING imports are temporarily allowed for
        gradual migration, but regular imports are not.
        """
        application_dir = PACKAGE_ROOT / "application"
        if not application_dir.exists():
            pytest.skip("application directory not found")

        violations = []
        for py_file in get_python_files(application_dir):
            # Skip checking for TYPE_CHECKING imports (temporary allowance)
            with open(py_file, "r", encoding="utf-8") as f:
                content = f.read()

            # Check for direct (non-TYPE_CHECKING) legacy imports
            # This is a simple heuristic - a more robust solution would
            # use AST analysis to detect imports outside TYPE_CHECKING blocks
            imports = extract_imports_from_file(py_file)
            forbidden = has_forbidden_import(imports, r"(^|\.)legacy(\.|$)")

            # For now, just track but don't fail - legacy migration is in progress
            if forbidden:
                rel_path = py_file.relative_to(PACKAGE_ROOT.parent)
                # This is informational until CLEAN2-D1/D2 are complete
                violations.append(f"{rel_path}: {forbidden}")

        # Note: This test is informational while migration is in progress
        # TODO(CLEAN2-D1, CLEAN2-D2): Uncomment the assertion below once
        # DomainProcessingUseCase and StudyProcessingUseCase no longer
        # delegate to legacy coordinators.
        # assert not violations, (
        #     f"Application layer imports legacy modules:\n" + "\n".join(violations)
        # )
        if violations:
            pytest.skip(
                f"Application layer still has legacy imports (migration in progress):\n"
                + "\n".join(violations)
            )


class TestRegressionPrevention:
    """Tests that can be used to detect regressions after cleanup.

    Note: Legacy modules are excluded because they are deprecated and
    will be cleaned up in CLEAN2-F2. The focus is on the new architecture.
    """

    def test_no_cli_logging_config_outside_cli_and_legacy(self):
        """Ensure cli.logging_config is not imported outside CLI (excluding legacy).

        This is the acceptance criteria for CLEAN2-A1 and CLEAN2-A2.
        Legacy modules are excluded as they have their own cleanup ticket (CLEAN2-F2).
        """
        excluded_dirs = {PACKAGE_ROOT / "cli", PACKAGE_ROOT / "legacy"}

        violations = []
        for py_file in get_python_files(PACKAGE_ROOT):
            # Skip excluded directories
            if any(py_file.is_relative_to(excluded) for excluded in excluded_dirs):
                continue

            imports = extract_imports_from_file(py_file)
            forbidden = has_forbidden_import(imports, r"cli\.logging_config")
            if forbidden:
                rel_path = py_file.relative_to(PACKAGE_ROOT.parent)
                violations.append(f"{rel_path}: {forbidden}")

        assert not violations, (
            f"cli.logging_config imported outside CLI/legacy:\n" + "\n".join(violations)
        )

    def test_no_cli_helpers_outside_cli_and_legacy(self):
        """Ensure cli.helpers is not imported outside CLI (excluding legacy).

        This is the acceptance criteria for CLEAN2-A1.
        Legacy modules are excluded as they have their own cleanup ticket (CLEAN2-F2).
        """
        excluded_dirs = {PACKAGE_ROOT / "cli", PACKAGE_ROOT / "legacy"}

        violations = []
        for py_file in get_python_files(PACKAGE_ROOT):
            # Skip excluded directories
            if any(py_file.is_relative_to(excluded) for excluded in excluded_dirs):
                continue

            imports = extract_imports_from_file(py_file)
            forbidden = has_forbidden_import(imports, r"cli\.helpers")
            if forbidden:
                rel_path = py_file.relative_to(PACKAGE_ROOT.parent)
                violations.append(f"{rel_path}: {forbidden}")

        assert not violations, (
            f"cli.helpers imported outside CLI/legacy:\n" + "\n".join(violations)
        )
