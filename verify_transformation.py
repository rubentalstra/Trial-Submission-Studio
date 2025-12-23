from pathlib import Path
import xml.etree.ElementTree as ET

import pandas as pd


def parse_dataset_xml(xml_path):
    """Parses Dataset-XML and returns a DataFrame."""
    try:
        tree = ET.parse(xml_path)
        root = tree.getroot()
        # Namespaces are annoying in ElementTree.
        # We'll just ignore them by using local-name() in xpath if possible,
        # or just strip them from tags.
        # But findall with {ns}tag works.

        data = []
        # Find ItemGroupData (rows)
        # Path: ODM/ClinicalData/ItemGroupData
        # Note: Namespaces might be tricky, usually default or specific.
        # Let's try finding all ItemGroupData regardless of namespace or path
        for group in root.findall(".//{http://www.cdisc.org/ns/odm/v1.3}ItemGroupData"):
            row = {}
            for item in group.findall("{http://www.cdisc.org/ns/odm/v1.3}ItemData"):
                oid = item.get("ItemOID")
                val = item.get("Value")
                # OID is usually IT.DOMAIN.VAR, e.g. IT.VS.VSORRES
                if oid:
                    var_name = oid.split(".")[-1]
                    row[var_name] = val
            data.append(row)

        return pd.DataFrame(data)
    except Exception as e:
        print(f"Error parsing XML {xml_path}: {e}")
        return pd.DataFrame()


def verify_vs():
    print("--- Verifying VS Domain ---")
    source_path = Path(
        "mockdata/DEMO_GDISC_20240903_072908/DEMO_GDISC_20240903_072908_VS.csv"
    )
    xml_path = Path("mockdata/DEMO_GDISC_20240903_072908/output/dataset-xml/vs.xml")
    supp_path = Path(
        "mockdata/DEMO_GDISC_20240903_072908/output/dataset-xml/suppvs.xml"
    )

    # Read CSV with header=1 (second row)
    source_df = pd.read_csv(source_path, header=1)
    print(f"Source VS columns (row 2): {list(source_df.columns[:5])} ...")

    expected_cols = [c for c in source_df.columns if c.startswith("ORRES_")]
    print(f"Found wide columns in source: {expected_cols}")

    # Load XML
    vs_df = parse_dataset_xml(xml_path)
    print(f"Loaded VS XML: {len(vs_df)} rows")
    if vs_df.empty:
        print("❌ VS XML is empty or failed to parse")
        return

    unique_tests = vs_df["VSTESTCD"].unique() if "VSTESTCD" in vs_df.columns else []
    print(f"Unique VSTESTCD in XML: {unique_tests}")

    # Check for data presence
    for col in expected_cols:
        testcd = col.replace("ORRES_", "")
        if testcd in unique_tests:
            print(f"✅ Found records for {testcd}")
            # Optional: Check a value
            # Find a non-empty value in source
            sample = (
                source_df[source_df[col].notna()].iloc[0]
                if not source_df[source_df[col].notna()].empty
                else None
            )
            if sample is not None:
                src_val = str(sample[col])
                # Find corresponding record in VS (simplified check)
                # We assume USUBJID matches SubjectId (with prefix)
                # But let's just check if the value exists in VSORRES for that test
                matches = vs_df[
                    (vs_df["VSTESTCD"] == testcd) & (vs_df["VSORRES"] == src_val)
                ]
                if not matches.empty:
                    print(f"   ✅ Value '{src_val}' found in VSORRES")
                else:
                    print(
                        f"   ⚠️ Value '{src_val}' NOT found in VSORRES (might be formatting diff)"
                    )
        else:
            print(f"❌ Missing records for {testcd}")

    # Verify SUPPVS
    if supp_path.exists():
        supp_df = parse_dataset_xml(supp_path)
        print(f"Loaded SUPPVS XML: {len(supp_df)} rows")
        if not supp_df.empty:
            qnams = supp_df["QNAM"].unique() if "QNAM" in supp_df.columns else []
            print(f"QNAMs in SUPPVS: {qnams}")

            leaked = [col for col in expected_cols if col in qnams]
            # Also check sanitized names (max 8 chars)
            # sanitize_qnam logic: alphanumeric, max 8.
            # ORRES_HEIGHT -> ORRES_HE (8 chars)
            # But the leakage usually preserves the full name if it fits or truncates.
            # Let's check if any QNAM starts with ORRES
            leaked_starts = [q for q in qnams if str(q).startswith("ORRES")]

            if leaked:
                print(f"❌ LEAKAGE DETECTED: Source columns found in SUPPVS: {leaked}")
            elif leaked_starts:
                print(
                    f"❌ LEAKAGE DETECTED: QNAMs starting with ORRES found: {leaked_starts}"
                )
            else:
                print("✅ No source columns leaked into SUPPVS")
        else:
            print("✅ SUPPVS is empty (Good)")


def verify_lb():
    print("\n--- Verifying LB Domain ---")
    source_path = Path(
        "mockdata/DEMO_GDISC_20240903_072908/DEMO_GDISC_20240903_072908_LBHM.csv"
    )
    xml_path = Path("mockdata/DEMO_GDISC_20240903_072908/output/dataset-xml/lb.xml")
    supp_path = Path(
        "mockdata/DEMO_GDISC_20240903_072908/output/dataset-xml/supplb.xml"
    )  # Might not exist if merged into supplb? No, supplb is the file.

    source_df = pd.read_csv(source_path, header=1)
    expected_cols = [c for c in source_df.columns if c.startswith("ORRES_")]
    print(f"Found wide columns in source LBHM: {expected_cols}")

    lb_df = parse_dataset_xml(xml_path)
    print(f"Loaded LB XML: {len(lb_df)} rows")

    unique_tests = lb_df["LBTESTCD"].unique() if "LBTESTCD" in lb_df.columns else []
    print(f"Unique LBTESTCD in XML: {unique_tests}")

    for col in expected_cols:
        testcd = col.replace("ORRES_", "")
        if testcd in unique_tests:
            print(f"✅ Found records for {testcd}")
        # Some might be filtered if empty
        elif source_df[col].notna().any():
            print(f"❌ Missing records for {testcd} (Source has data)")
        else:
            print(f"ℹ️ No records for {testcd} (Source is empty)")

    if supp_path.exists():
        supp_df = parse_dataset_xml(supp_path)
        # Note: LB is merged from multiple files, so SUPPLB might contain QNAMs from other files.
        # We check for leakage of OUR expected columns.
        if not supp_df.empty:
            qnams = supp_df["QNAM"].unique() if "QNAM" in supp_df.columns else []
            leaked = [col for col in expected_cols if col in qnams]
            leaked_starts = [q for q in qnams if str(q).startswith("ORRES")]

            if leaked:
                print(f"❌ LEAKAGE DETECTED: Source columns found in SUPPLB: {leaked}")
            elif leaked_starts:
                print(
                    f"❌ LEAKAGE DETECTED: QNAMs starting with ORRES found: {leaked_starts}"
                )
            else:
                print("✅ No source columns leaked into SUPPLB")


if __name__ == "__main__":
    verify_vs()
    verify_lb()
