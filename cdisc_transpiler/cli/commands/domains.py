import click
from rich.console import Console
from rich.table import Table

from ...infrastructure.sdtm_spec.registry import (
    get_domain,
    list_domains as list_all_domains,
)

console = Console()


@click.command()
def list_domains_command() -> None:
    table = Table(title="Supported SDTM Domains")
    table.add_column("Code", style="cyan")
    table.add_column("Description")
    for code in list_all_domains():
        domain = get_domain(code)
        table.add_row(domain.code, domain.description)
    console.print(table)
