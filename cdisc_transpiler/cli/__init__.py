import click

from .commands.domains import list_domains_command
from .commands.study import study_command


@click.group()
def app() -> None:
    pass


app.add_command(study_command, name="study")
app.add_command(list_domains_command, name="domains")
__all__ = ["app"]
