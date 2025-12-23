"""Progress presenter for domain processing.

This module provides the ProgressPresenter class that tracks and displays
progress during study processing operations.
"""

from rich.console import Console


class ProgressPresenter:
    """Presenter for tracking and displaying domain processing progress.

    This class provides a clean interface for tracking progress through
    domain processing, including success, error, and warning counts. It
    follows the presenter pattern by separating progress tracking logic
    from business logic.

    Attributes:
        console: Rich console for output
        total_domains: Total number of domains to process
        processed: Number of domains processed so far
        errors: Number of domains that failed
        warnings: Number of domains with warnings

    Example:
        >>> console = Console()
        >>> presenter = ProgressPresenter(console, total_domains=10)
        >>> presenter.increment()
        >>> presenter.increment(error=True)
        >>> presenter.print_summary()
    """

    def __init__(self, console: Console, total_domains: int):
        """Initialize the progress presenter.

        Args:
            console: Rich console for output
            total_domains: Total number of domains to process
        """
        super().__init__()
        self.console = console
        self.total_domains = total_domains
        self.processed = 0
        self.errors = 0
        self.warnings = 0

    def increment(self, *, error: bool = False, warning: bool = False) -> None:
        """Increment progress counters.

        This method should be called after each domain is processed to
        update the progress counters.

        Args:
            error: Whether this domain had errors
            warning: Whether this domain had warnings

        Example:
            >>> presenter.increment()  # Success
            >>> presenter.increment(error=True)  # Failed
            >>> presenter.increment(warning=True)  # Warning
        """
        self.processed += 1
        if error:
            self.errors += 1
        if warning:
            self.warnings += 1

    @property
    def success_count(self) -> int:
        """Get the number of successfully processed domains.

        Returns:
            Number of domains processed without errors
        """
        return self.processed - self.errors

    @property
    def is_complete(self) -> bool:
        """Check if all domains have been processed.

        Returns:
            True if all domains have been processed
        """
        return self.processed >= self.total_domains

    @property
    def progress_percentage(self) -> float:
        """Calculate the progress percentage.

        Returns:
            Progress as a percentage (0.0 to 100.0)
        """
        if self.total_domains == 0:
            return 100.0
        return (self.processed / self.total_domains) * 100.0

    def print_summary(self) -> None:
        """Print a summary of progress.

        Displays the current progress including total processed, success
        count, errors, and warnings using Rich formatting.

        Example:
            >>> presenter.print_summary()
            Progress:
              Processed: 8/10
              Success: 7
              Errors: 1
        """
        self.console.print("\n[bold]Progress:[/bold]")
        self.console.print(f"  Processed: {self.processed}/{self.total_domains}")
        self.console.print(f"  [green]Success: {self.success_count}[/green]")
        if self.errors:
            self.console.print(f"  [red]Errors: {self.errors}[/red]")
        if self.warnings:
            self.console.print(f"  [yellow]Warnings: {self.warnings}[/yellow]")

    def print_progress_line(self) -> None:
        """Print a single-line progress indicator.

        Displays a compact progress line showing the current status.
        Useful for live updates during processing.

        Example:
            >>> presenter.print_progress_line()
            [Processing 5/10 domains... ✓ 4 ✗ 1]
        """
        status_parts: list[str] = []
        if self.success_count > 0:
            status_parts.append(f"[green]✓ {self.success_count}[/green]")
        if self.errors > 0:
            status_parts.append(f"[red]✗ {self.errors}[/red]")
        if self.warnings > 0:
            status_parts.append(f"[yellow]⚠ {self.warnings}[/yellow]")

        status = " ".join(status_parts) if status_parts else ""
        self.console.print(
            f"[dim]Processing {self.processed}/{self.total_domains} domains... {status}[/dim]"
        )

    def reset(self) -> None:
        """Reset all progress counters.

        Useful if you need to reuse the same presenter for a new batch.
        """
        self.processed = 0
        self.errors = 0
        self.warnings = 0
