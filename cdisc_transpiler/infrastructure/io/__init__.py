"""Infrastructure I/O layer.

This package contains adapters and concrete implementations for reading and
writing files (CSV, XPT, Dataset-XML, Define-XML, SAS).

Architecture note:
- Avoid re-exporting symbols from here; import from the defining modules.
- Application DTOs live in cdisc_transpiler.application.models.
"""
