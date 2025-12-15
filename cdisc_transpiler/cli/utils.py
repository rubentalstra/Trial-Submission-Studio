"""Progress tracking utilities for CLI.

This module maintains backward compatibility by re-exporting ProgressPresenter
as ProgressTracker. New code should import directly from presenters.progress.
"""

from __future__ import annotations

from rich.console import Console

from .presenters.progress import ProgressPresenter

console = Console()

# Backward compatibility: ProgressTracker is now an alias for ProgressPresenter
ProgressTracker = ProgressPresenter
