"""Progress tracking utilities for CLI.

Provides clean progress reporting with Rich console.
"""

from __future__ import annotations

from contextlib import contextmanager
from typing import TYPE_CHECKING

from rich.console import Console
from rich.progress import (
    Progress,
    SpinnerColumn,
    TextColumn,
    BarColumn,
    TaskProgressColumn,
    TimeRemainingColumn,
)

if TYPE_CHECKING:
    from pathlib import Path

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
        console.print(f"\n[bold]Progress:[/bold]")
        console.print(f"  Processed: {self.processed}/{self.total_domains}")
        console.print(f"  [green]Success: {self.success_count}[/green]")
        if self.errors:
            console.print(f"  [red]Errors: {self.errors}[/red]")
        if self.warnings:
            console.print(f"  [yellow]Warnings: {self.warnings}[/yellow]")


@contextmanager
def progress_bar(description: str, total: int | None = None):
    """Create a progress bar context manager.
    
    Args:
        description: Task description
        total: Total steps (None for indeterminate)
        
    Yields:
        Progress task
    """
    with Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        BarColumn(),
        TaskProgressColumn(),
        TimeRemainingColumn(),
        console=console,
    ) as progress:
        task = progress.add_task(description, total=total)
        yield progress, task


def log_info(message: str) -> None:
    """Log info message."""
    console.print(f"[dim]{message}[/dim]")


def log_success(message: str) -> None:
    """Log success message."""
    console.print(f"[green]✓[/green] {message}")


def log_warning(message: str) -> None:
    """Log warning message."""
    console.print(f"[yellow]⚠[/yellow] {message}")


def log_error(message: str) -> None:
    """Log error message."""
    console.print(f"[red]✗[/red] {message}")


def print_domain_header(domain_code: str, files: list[Path]) -> None:
    """Print domain processing header.
    
    Args:
        domain_code: Domain code
        files: List of input files
    """
    if len(files) == 1:
        console.print(f"\n[bold]Processing {domain_code}[/bold]")
    else:
        console.print(f"\n[bold]Processing {domain_code}[/bold] (merging {len(files)} files)")
    
    for file_path in files:
        console.print(f"  - {file_path.name}")


def print_file_generated(file_type: str, path: Path) -> None:
    """Print file generation success.
    
    Args:
        file_type: Type of file (XPT, XML, SAS)
        path: Path to generated file
    """
    log_success(f"Generated {file_type}: {path}")
