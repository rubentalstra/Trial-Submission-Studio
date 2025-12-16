"""Unit tests for ProgressPresenter class."""

import pytest
from io import StringIO
from rich.console import Console

from cdisc_transpiler.cli.presenters import ProgressPresenter


class TestProgressPresenter:
    """Test suite for ProgressPresenter class."""

    @pytest.fixture
    def console(self):
        """Create a console with StringIO for capturing output."""
        return Console(file=StringIO(), force_terminal=True, width=120)

    @pytest.fixture
    def presenter(self, console):
        """Create a ProgressPresenter instance."""
        return ProgressPresenter(console, total_domains=10)

    def test_initialization(self, console):
        """Test that presenter initializes with correct values."""
        presenter = ProgressPresenter(console, total_domains=5)
        assert presenter.console == console
        assert presenter.total_domains == 5
        assert presenter.processed == 0
        assert presenter.errors == 0
        assert presenter.warnings == 0

    def test_increment_success(self, presenter):
        """Test incrementing without errors or warnings."""
        presenter.increment()
        assert presenter.processed == 1
        assert presenter.errors == 0
        assert presenter.warnings == 0
        assert presenter.success_count == 1

    def test_increment_error(self, presenter):
        """Test incrementing with error."""
        presenter.increment(error=True)
        assert presenter.processed == 1
        assert presenter.errors == 1
        assert presenter.warnings == 0
        assert presenter.success_count == 0

    def test_increment_warning(self, presenter):
        """Test incrementing with warning."""
        presenter.increment(warning=True)
        assert presenter.processed == 1
        assert presenter.errors == 0
        assert presenter.warnings == 1
        assert presenter.success_count == 1

    def test_increment_error_and_warning(self, presenter):
        """Test incrementing with both error and warning."""
        presenter.increment(error=True, warning=True)
        assert presenter.processed == 1
        assert presenter.errors == 1
        assert presenter.warnings == 1
        assert presenter.success_count == 0

    def test_multiple_increments(self, presenter):
        """Test multiple increments with mixed results."""
        presenter.increment()  # Success
        presenter.increment()  # Success
        presenter.increment(error=True)  # Error
        presenter.increment(warning=True)  # Warning
        presenter.increment(error=True, warning=True)  # Both

        assert presenter.processed == 5
        assert presenter.errors == 2
        assert presenter.warnings == 2
        assert presenter.success_count == 3

    def test_success_count_property(self, presenter):
        """Test that success_count is calculated correctly."""
        presenter.increment()
        presenter.increment()
        presenter.increment(error=True)

        assert presenter.success_count == 2
        assert presenter.processed == 3
        assert presenter.errors == 1

    def test_is_complete_false(self, presenter):
        """Test is_complete when processing is not done."""
        presenter.increment()
        presenter.increment()
        assert not presenter.is_complete

    def test_is_complete_true(self, presenter):
        """Test is_complete when all domains processed."""
        for _ in range(10):
            presenter.increment()
        assert presenter.is_complete

    def test_is_complete_exceeds(self, presenter):
        """Test is_complete when processed exceeds total."""
        for _ in range(11):
            presenter.increment()
        assert presenter.is_complete

    def test_progress_percentage_zero(self, console):
        """Test progress percentage at start."""
        presenter = ProgressPresenter(console, total_domains=10)
        assert presenter.progress_percentage == 0.0

    def test_progress_percentage_half(self, console):
        """Test progress percentage at halfway point."""
        presenter = ProgressPresenter(console, total_domains=10)
        for _ in range(5):
            presenter.increment()
        assert presenter.progress_percentage == 50.0

    def test_progress_percentage_complete(self, console):
        """Test progress percentage when complete."""
        presenter = ProgressPresenter(console, total_domains=10)
        for _ in range(10):
            presenter.increment()
        assert presenter.progress_percentage == 100.0

    def test_progress_percentage_zero_domains(self, console):
        """Test progress percentage with zero domains."""
        presenter = ProgressPresenter(console, total_domains=0)
        assert presenter.progress_percentage == 100.0

    def test_print_summary_no_errors(self, presenter, console):
        """Test print_summary with no errors or warnings."""
        presenter.increment()
        presenter.increment()
        presenter.print_summary()

        output = console.file.getvalue()
        # Remove ANSI codes for testing
        import re

        output_clean = re.sub(r"\x1b\[[0-9;]*m", "", output)

        assert "Progress:" in output_clean
        assert "Processed: 2/10" in output_clean
        assert "Success: 2" in output_clean
        assert "Errors:" not in output_clean
        assert "Warnings:" not in output_clean

    def test_print_summary_with_errors(self, presenter, console):
        """Test print_summary with errors."""
        presenter.increment()
        presenter.increment(error=True)
        presenter.print_summary()

        output = console.file.getvalue()
        import re

        output_clean = re.sub(r"\x1b\[[0-9;]*m", "", output)

        assert "Processed: 2/10" in output_clean
        assert "Success: 1" in output_clean
        assert "Errors: 1" in output_clean

    def test_print_summary_with_warnings(self, presenter, console):
        """Test print_summary with warnings."""
        presenter.increment()
        presenter.increment(warning=True)
        presenter.print_summary()

        output = console.file.getvalue()
        import re

        output_clean = re.sub(r"\x1b\[[0-9;]*m", "", output)

        assert "Processed: 2/10" in output_clean
        assert "Success: 2" in output_clean
        assert "Warnings: 1" in output_clean

    def test_print_summary_with_all(self, presenter, console):
        """Test print_summary with successes, errors, and warnings."""
        presenter.increment()
        presenter.increment(error=True)
        presenter.increment(warning=True)
        presenter.print_summary()

        output = console.file.getvalue()
        import re

        output_clean = re.sub(r"\x1b\[[0-9;]*m", "", output)

        assert "Processed: 3/10" in output_clean
        assert "Success: 2" in output_clean
        assert "Errors: 1" in output_clean
        assert "Warnings: 1" in output_clean

    def test_print_progress_line_empty(self, presenter, console):
        """Test print_progress_line with no progress."""
        presenter.print_progress_line()

        output = console.file.getvalue()
        import re

        output_clean = re.sub(r"\x1b\[[0-9;]*m", "", output)
        assert "Processing 0/10 domains" in output_clean

    def test_print_progress_line_with_success(self, presenter, console):
        """Test print_progress_line with successful domains."""
        presenter.increment()
        presenter.increment()
        presenter.print_progress_line()

        output = console.file.getvalue()
        import re

        output_clean = re.sub(r"\x1b\[[0-9;]*m", "", output)

        assert "Processing 2/10 domains" in output_clean
        assert "✓ 2" in output_clean

    def test_print_progress_line_with_errors(self, presenter, console):
        """Test print_progress_line with errors."""
        presenter.increment()
        presenter.increment(error=True)
        presenter.print_progress_line()

        output = console.file.getvalue()
        import re

        output_clean = re.sub(r"\x1b\[[0-9;]*m", "", output)

        assert "Processing 2/10 domains" in output_clean
        assert "✓ 1" in output_clean
        assert "✗ 1" in output_clean

    def test_print_progress_line_with_warnings(self, presenter, console):
        """Test print_progress_line with warnings."""
        presenter.increment()
        presenter.increment(warning=True)
        presenter.print_progress_line()

        output = console.file.getvalue()
        import re

        output_clean = re.sub(r"\x1b\[[0-9;]*m", "", output)

        assert "Processing 2/10 domains" in output_clean
        assert "✓ 2" in output_clean
        assert "⚠ 1" in output_clean

    def test_reset(self, presenter):
        """Test reset functionality."""
        presenter.increment()
        presenter.increment(error=True)
        presenter.increment(warning=True)

        assert presenter.processed == 3
        assert presenter.errors == 1
        assert presenter.warnings == 1

        presenter.reset()

        assert presenter.processed == 0
        assert presenter.errors == 0
        assert presenter.warnings == 0
        assert presenter.success_count == 0


class TestProgressPresenterIntegration:
    """Integration tests for ProgressPresenter."""

    @pytest.fixture
    def console(self):
        """Create a console with StringIO for capturing output."""
        return Console(file=StringIO(), force_terminal=True, width=120)

    def test_realistic_workflow(self, console):
        """Test a realistic processing workflow."""
        presenter = ProgressPresenter(console, total_domains=5)

        # Process domains with mixed results
        presenter.increment()  # Domain 1: success
        presenter.increment()  # Domain 2: success
        presenter.increment(error=True)  # Domain 3: error
        presenter.increment(warning=True)  # Domain 4: warning
        presenter.increment()  # Domain 5: success

        assert presenter.is_complete
        assert presenter.processed == 5
        assert presenter.success_count == 4
        assert presenter.errors == 1
        assert presenter.warnings == 1
        assert presenter.progress_percentage == 100.0

    def test_reuse_presenter(self, console):
        """Test reusing presenter after reset."""
        presenter = ProgressPresenter(console, total_domains=3)

        # First batch
        for _ in range(3):
            presenter.increment()
        assert presenter.is_complete

        # Reset and process again
        presenter.reset()
        assert not presenter.is_complete
        assert presenter.processed == 0

        # Second batch
        presenter.increment()
        presenter.increment(error=True)
        assert presenter.processed == 2
        assert presenter.success_count == 1
