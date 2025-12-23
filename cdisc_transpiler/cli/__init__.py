"""CLI package for CDISC Transpiler.

This package provides a modular command-line interface for the CDISC Transpiler.
Commands are organized into separate modules for better maintainability.
"""

import click

from .commands.domains import list_domains_command
from .commands.study import study_command


@click.group()
def app() -> None:
    """CDISC Transpiler CLI - Generate SDTM submission files from study data."""
    pass


# Register commands
app.add_command(study_command, name="study")
app.add_command(list_domains_command, name="domains")


__all__ = ["app"]
