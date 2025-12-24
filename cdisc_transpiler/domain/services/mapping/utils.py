import re

_SAS_NAME_LITERAL_RE = re.compile('^(?P<quoted>"(?:[^"]|"")*")n$', re.IGNORECASE)


def normalize_text(text: str) -> str:
    return re.sub("[^A-Z0-9]", "", text.upper())


def safe_column_name(column: str) -> str:
    if re.match("^[A-Za-z_][A-Za-z0-9_]*$", column):
        return column
    escaped = column.replace('"', '""')
    return f'"{escaped}"n'


def unquote_column_name(name: str | None) -> str:
    if not name:
        return ""
    name_str = str(name)
    match = _SAS_NAME_LITERAL_RE.fullmatch(name_str)
    if not match:
        return name_str
    quoted = match.group("quoted")
    return quoted[1:-1].replace('""', '"')
