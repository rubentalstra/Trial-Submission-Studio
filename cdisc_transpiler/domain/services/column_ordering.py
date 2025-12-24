from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import pandas as pd

    from ..entities.sdtm_domain import SDTMDomain


def ordered_columns_for_domain(
    dataset: pd.DataFrame, *, domain: SDTMDomain
) -> list[str]:
    dataset_columns = [str(c) for c in dataset.columns]
    present_upper = {c.upper() for c in dataset_columns}
    spec_order_upper: list[str] = []
    for var in domain.variables:
        upper = var.name.upper()
        if upper in present_upper:
            spec_order_upper.append(upper)
    canonical_prefix = [
        c for c in ("STUDYID", "DOMAIN", "USUBJID") if c in present_upper
    ]
    seq_cols = [c for c in spec_order_upper if c.endswith("SEQ")]
    canonical = canonical_prefix + [c for c in seq_cols if c not in canonical_prefix]
    spec_order_upper = canonical + [c for c in spec_order_upper if c not in canonical]
    by_upper = {c.upper(): c for c in dataset_columns}
    spec_set = set(spec_order_upper)
    spec_iter = iter(spec_order_upper)
    ordered: list[str] = []
    for col in dataset_columns:
        if col.upper() in spec_set:
            try:
                next_upper = next(spec_iter)
            except StopIteration:
                ordered.append(col)
                continue
            ordered.append(by_upper.get(next_upper, col))
        else:
            ordered.append(col)
    return ordered
