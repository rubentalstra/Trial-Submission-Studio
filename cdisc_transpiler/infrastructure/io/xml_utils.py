from pathlib import Path


def tag(namespace: str, name: str) -> str:
    return f"{{{namespace}}}{name}"


def attr(namespace: str, name: str) -> str:
    return f"{{{namespace}}}{name}"


def safe_href(href: str) -> str:
    if not href:
        return href
    path = Path(href)
    stem = path.stem[:8]
    new_name = f"{stem}{path.suffix}".lower()
    safe = str(path.with_name(new_name))
    return safe[:64]
