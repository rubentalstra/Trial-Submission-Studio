"""Presenters for CLI output formatting.

This module contains presenter classes that format and display information
to the user via the CLI. Presenters are responsible for formatting data
structures into human-readable output.
"""

from .summary import SummaryPresenter

__all__ = ["SummaryPresenter"]
