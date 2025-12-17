"""Deterministic SDTM column ordering.

SDTMIG v3.4 Section 4.1.4 states that datasets (and Define-XML) must reflect a
consistent SDTM variable ordering, with Identifier variables first, followed by
Topic/Qualifier/Timing variables.

This module provides a conservative ordering helper:
- Only reorders columns that are present in the SDTMIG domain specification.
- Leaves non-standard (sponsor) columns anchored in their original positions.
"""

from __future__ import annotations

import pandas as pd


def ordered_columns_for_domain(dataset: pd.DataFrame, *, domain: object) -> list[str]:
    """Return dataset columns ordered per SDTMIG spec for the given domain.

    Important: do NOT move unknown/sponsor columns.
    Only reorders the subset of columns known to the SDTMIG spec, while leaving
    any non-spec columns anchored in their original positions.
    """

    dataset_columns = [str(c) for c in dataset.columns]
    present_upper = {c.upper() for c in dataset_columns}

    spec_order_upper: list[str] = []
    domain_vars = getattr(domain, "variables", None) or []
    for var in domain_vars:
        name = getattr(var, "name", None)
        if not name:
            continue
        upper = str(name).upper()
        if upper in present_upper:
            spec_order_upper.append(upper)

    # Many validators expect the standard SDTM identifier columns to appear
    # first; enforce a canonical prefix even if the domain spec CSV ordering is
    # odd.
    canonical_prefix = [
        c for c in ("STUDYID", "DOMAIN", "USUBJID") if c in present_upper
    ]
    seq_cols = [c for c in spec_order_upper if c.endswith("SEQ")]
    canonical = canonical_prefix + [c for c in seq_cols if c not in canonical_prefix]
    spec_order_upper = canonical + [c for c in spec_order_upper if c not in canonical]

    # Preserve original casing of the incoming dataset columns.
    by_upper = {c.upper(): c for c in dataset_columns}
    spec_set = set(spec_order_upper)
    spec_iter = iter(spec_order_upper)

    ordered: list[str] = []
    for col in dataset_columns:
        if col.upper() in spec_set:
            try:
                next_upper = next(spec_iter)
            except StopIteration:  # pragma: no cover
                ordered.append(col)
                continue
            ordered.append(by_upper.get(next_upper, col))
        else:
            ordered.append(col)

    return ordered
