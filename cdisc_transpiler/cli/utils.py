"""Progress tracking utilities for CLI.

Provides clean progress reporting with Rich console.
"""

from __future__ import annotations

from rich.console import Console

console = Console()


class ProgressTracker:
    """Track progress of domain processing."""

    def __init__(self, total_domains: int):
        """Initialize progress tracker.

        Args:
            total_domains: Total number of domains to process
        """
        self.total_domains = total_domains
        self.processed = 0
        self.errors = 0
        self.warnings = 0

    def increment(self, *, error: bool = False, warning: bool = False) -> None:
        """Increment progress counters.

        Args:
            error: Whether this domain had errors
            warning: Whether this domain had warnings
        """
        self.processed += 1
        if error:
            self.errors += 1
        if warning:
            self.warnings += 1

    @property
    def success_count(self) -> int:
        """Number of successful domains."""
        return self.processed - self.errors

    def print_summary(self) -> None:
        """Print progress summary."""
        console.print("\n[bold]Progress:[/bold]")
        console.print(f"  Processed: {self.processed}/{self.total_domains}")
        console.print(f"  [green]Success: {self.success_count}[/green]")
        if self.errors:
            console.print(f"  [red]Errors: {self.errors}[/red]")
        if self.warnings:
            console.print(f"  [yellow]Warnings: {self.warnings}[/yellow]")


def log_success(message: str) -> None:
    """Log success message."""
    console.print(f"[green]✓[/green] {message}")


def log_warning(message: str) -> None:
    """Log warning message."""
    console.print(f"[yellow]⚠[/yellow] {message}")


def log_error(message: str) -> None:
    """Log error message."""
    console.print(f"[red]✗[/red] {message}")
