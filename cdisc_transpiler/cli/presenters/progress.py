from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from rich.console import Console


class ProgressPresenter:
    pass

    def __init__(self, console: Console, total_domains: int) -> None:
        super().__init__()
        self.console = console
        self.total_domains = total_domains
        self.processed = 0
        self.errors = 0
        self.warnings = 0

    def increment(self, *, error: bool = False, warning: bool = False) -> None:
        self.processed += 1
        if error:
            self.errors += 1
        if warning:
            self.warnings += 1

    @property
    def success_count(self) -> int:
        return self.processed - self.errors

    @property
    def is_complete(self) -> bool:
        return self.processed >= self.total_domains

    @property
    def progress_percentage(self) -> float:
        if self.total_domains == 0:
            return 100.0
        return self.processed / self.total_domains * 100.0

    def print_summary(self) -> None:
        self.console.print("\n[bold]Progress:[/bold]")
        self.console.print(f"  Processed: {self.processed}/{self.total_domains}")
        self.console.print(f"  [green]Success: {self.success_count}[/green]")
        if self.errors:
            self.console.print(f"  [red]Errors: {self.errors}[/red]")
        if self.warnings:
            self.console.print(f"  [yellow]Warnings: {self.warnings}[/yellow]")

    def print_progress_line(self) -> None:
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
        self.processed = 0
        self.errors = 0
        self.warnings = 0
