"""CLI package for CDISC Transpiler.

This package provides a modular command-line interface for the CDISC Transpiler.
Commands are organized into separate modules for better maintainability.
"""

from __future__ import annotations

import click

from .commands import study, domains


@click.group()
def app() -> None:
    """CDISC Transpiler CLI - Generate SDTM submission files from study data."""
    pass


# Register commands
app.add_command(study.study_command, name="study")
app.add_command(domains.list_domains_command, name="domains")


__all__ = ["app"]
