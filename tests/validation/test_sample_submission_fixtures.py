"""Golden fixture validation against the CDISC MSG sample submission package.

These tests compare our infrastructure writers/generators against the official
sample outputs checked into:
- tests/validation/data/xpt
- tests/validation/data/Dataset-XML

The intent is to validate standard-compliant serialization (XPT and Dataset-XML)
without depending on the full end-to-end mapping pipeline.
"""

from collections.abc import Iterable
from pathlib import Path
import xml.etree.ElementTree as ET

import pandas.testing as pdt
import pyreadstat
import pytest

from cdisc_transpiler.domain.entities.mapping import MappingConfig
from cdisc_transpiler.infrastructure.io.dataset_xml_writer import (
    DATA_NS,
    DATASET_XML_VERSION,
    ODM_NS,
    write_dataset_xml,
)
from cdisc_transpiler.infrastructure.io.xpt_writer import XPTWriter

FIXTURES_DIR = Path(__file__).parent / "data"
FIXTURE_XPT_DIR = FIXTURES_DIR / "xpt"
FIXTURE_XML_DIR = FIXTURES_DIR / "Dataset-XML"


def _localname(tag: str) -> str:
    if "}" in tag:
        return tag.split("}", 1)[1]
    return tag


def _iter_elements(root: ET.Element) -> Iterable[ET.Element]:
    for elem in root.iter():
        if not isinstance(elem.tag, str):
            continue
        yield elem


def _find_first_item_group_oid(xml_path: Path) -> str:
    tree = ET.parse(xml_path)
    root = tree.getroot()
    for elem in _iter_elements(root):
        if _localname(elem.tag) == "ItemGroupData":
            item_group_oid = elem.attrib.get("ItemGroupOID")
            if item_group_oid:
                return item_group_oid
    raise AssertionError(f"No ItemGroupData/ItemGroupOID found in {xml_path}")


def _extract_domain_code_from_item_group_oid(item_group_oid: str) -> str:
    # Expected shape: IG.<DATASET_NAME>
    parts = item_group_oid.split(".")
    if len(parts) >= 2 and parts[0] == "IG":
        return parts[1].upper()
    raise AssertionError(f"Unexpected ItemGroupOID format: {item_group_oid}")


def _derive_domain_code_from_xpt(df) -> str | None:
    if "DOMAIN" not in df.columns or len(df) == 0:
        return None

    value = df["DOMAIN"].iloc[0]
    if value is None:
        return None
    text = str(value).strip().upper()
    return text or None


def _extract_item_group_data(xml_path: Path) -> list[tuple[str, list[tuple[str, str]]]]:
    """Return ordered list of ItemGroupData payloads.

    Each entry is:
      (ItemGroupOID, [(ItemOID, Value), ...])

    Notes:
    - Ignores comments and non-element nodes.
    - Ignores all other attributes (Seq, etc.).
    """

    tree = ET.parse(xml_path)
    root = tree.getroot()

    groups: list[tuple[str, list[tuple[str, str]]]] = []

    for item_group in _iter_elements(root):
        if _localname(item_group.tag) != "ItemGroupData":
            continue

        item_group_oid = item_group.attrib.get("ItemGroupOID")
        if not item_group_oid:
            raise AssertionError(f"Missing ItemGroupOID in {xml_path}")

        items: list[tuple[str, str]] = []
        for item_data in list(item_group):
            if not isinstance(item_data.tag, str):
                continue
            if _localname(item_data.tag) != "ItemData":
                continue
            item_oid = item_data.attrib.get("ItemOID")
            if item_oid is None:
                raise AssertionError(f"Missing ItemOID in {xml_path}")
            # In Dataset-XML, the value is in the Value attribute.
            value = item_data.attrib.get("Value", "")
            items.append((item_oid, value))

        groups.append((item_group_oid, items))

    if not groups:
        raise AssertionError(f"No ItemGroupData elements found in {xml_path}")

    return groups


def _collect_fixture_pairs() -> list[tuple[Path, Path]]:
    """Collect (xml_path, xpt_path) pairs from the fixture folders."""

    if not FIXTURE_XML_DIR.exists() or not FIXTURE_XPT_DIR.exists():
        return []

    pairs: list[tuple[Path, Path]] = []

    for xml_path in sorted(FIXTURE_XML_DIR.rglob("*.xml")):
        # Ignore non-dataset XML files (e.g., Define-XML or stylesheets) if present.
        if xml_path.name.lower().startswith("define"):
            continue

        rel = xml_path.relative_to(FIXTURE_XML_DIR)
        xpt_path = (FIXTURE_XPT_DIR / rel).with_suffix(".xpt")

        if xpt_path.exists():
            pairs.append((xml_path, xpt_path))

    return pairs


@pytest.mark.validation
def test_fixture_directories_present():
    assert FIXTURE_XPT_DIR.exists(), f"Missing fixture directory: {FIXTURE_XPT_DIR}"
    assert FIXTURE_XML_DIR.exists(), f"Missing fixture directory: {FIXTURE_XML_DIR}"


@pytest.mark.validation
@pytest.mark.parametrize("xml_path,xpt_path", _collect_fixture_pairs())
def test_dataset_xml_matches_official_fixture(
    xml_path: Path, xpt_path: Path, tmp_path: Path
):
    """Generate Dataset-XML from the official XPT and compare to official XML."""

    df, _meta = pyreadstat.read_xport(str(xpt_path))

    item_group_oid = _find_first_item_group_oid(xml_path)
    dataset_name = _extract_domain_code_from_item_group_oid(item_group_oid)
    domain_code = _derive_domain_code_from_xpt(df) or dataset_name

    config = MappingConfig(domain=domain_code, study_id="CDISCPILOT01", mappings=[])

    out_xml = tmp_path / xml_path.name
    write_dataset_xml(
        df,
        domain_code=domain_code,
        config=config,
        output=out_xml,
        dataset_name=dataset_name,
    )

    # Minimal structural assertions (avoid volatile fixture-specific attributes)
    out_tree = ET.parse(out_xml)
    out_root = out_tree.getroot()

    assert out_root.tag == f"{{{ODM_NS}}}ODM"
    assert out_root.attrib.get("FileType") == "Snapshot"
    assert out_root.attrib.get("ODMVersion") == "1.3.2"
    assert out_root.attrib.get(f"{{{DATA_NS}}}DatasetXMLVersion") == DATASET_XML_VERSION

    # Note: namespace declarations (xmlns:*) are not exposed via ElementTree's
    # parsed attributes, so we don't assert on them here.

    assert (out_root.attrib.get("FileOID") or "").strip()
    assert (out_root.attrib.get("PriorFileOID") or "").strip()

    containers = [
        e
        for e in list(out_root)
        if isinstance(e.tag, str)
        and _localname(e.tag) in {"ClinicalData", "ReferenceData"}
    ]
    assert len(containers) == 1
    container = containers[0]
    assert (container.attrib.get("StudyOID") or "").strip()
    assert (container.attrib.get("MetaDataVersionOID") or "").strip()

    # Ensure the sequence attribute is present and looks sane.
    seq_values: list[int] = []
    for elem in _iter_elements(out_root):
        if _localname(elem.tag) != "ItemGroupData":
            continue
        assert elem.attrib.get("ItemGroupOID") == f"IG.{dataset_name}"
        seq = elem.attrib.get(f"{{{DATA_NS}}}ItemGroupDataSeq")
        assert seq is not None
        assert seq.isdigit()
        seq_values.append(int(seq))
    assert seq_values == list(range(1, len(seq_values) + 1))

    # ItemOIDs must be dataset-prefixed (important for split datasets like QSSL).
    for elem in _iter_elements(out_root):
        if _localname(elem.tag) != "ItemData":
            continue
        item_oid = elem.attrib.get("ItemOID")
        assert item_oid is not None
        assert item_oid.startswith(f"IT.{dataset_name}.")

    expected_groups = _extract_item_group_data(xml_path)
    actual_groups = _extract_item_group_data(out_xml)

    assert actual_groups == expected_groups


def _collect_xpt_fixtures() -> list[Path]:
    if not FIXTURE_XPT_DIR.exists():
        return []

    return sorted(FIXTURE_XPT_DIR.rglob("*.xpt"))


@pytest.mark.validation
@pytest.mark.parametrize("xpt_path", _collect_xpt_fixtures())
def test_xpt_roundtrip_matches_official_data(xpt_path: Path, tmp_path: Path):
    """Round-trip the official XPT through our writer and compare data + key metadata."""

    df_expected, meta_expected = pyreadstat.read_xport(str(xpt_path))

    out_path = tmp_path / xpt_path.name
    writer = XPTWriter()

    # Preserve the original member name when re-writing.
    table_name = meta_expected.table_name or xpt_path.stem.upper()
    domain_code = _derive_domain_code_from_xpt(df_expected) or table_name[:2]
    writer.write(df_expected, domain_code, out_path, table_name=table_name)

    # Our writer enforces lowercase filenames on disk.
    out_path = out_path.with_name(out_path.name.lower())
    assert out_path.exists()

    df_actual, meta_actual = pyreadstat.read_xport(str(out_path))

    pdt.assert_frame_equal(df_actual, df_expected, check_dtype=False, check_like=False)

    # Table/member name is an important part of XPT compliance.
    assert (meta_actual.table_name or "") == table_name

    # Column labels should not be lost.
    assert getattr(meta_actual, "column_labels", None) is not None
    assert len(meta_actual.column_labels) == len(df_actual.columns)
