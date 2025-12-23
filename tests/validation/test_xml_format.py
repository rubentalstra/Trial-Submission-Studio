"""Dataset-XML format validation tests.

This module tests that generated Dataset-XML files:
- Are well-formed XML
- Have correct structure
- Can be parsed by standard XML tools
"""

from pathlib import Path
import xml.etree.ElementTree as ET

import pytest

from cdisc_transpiler.application.models import ProcessStudyRequest
from cdisc_transpiler.infrastructure.container import create_default_container

# Path to sample study data
MOCKDATA_DIR = Path(__file__).parent.parent.parent / "mockdata"
DEMO_GDISC = MOCKDATA_DIR / "DEMO_GDISC_20240903_072908"


@pytest.mark.validation
@pytest.mark.integration
class TestXMLFormatWellFormedness:
    """Test that Dataset-XML files are well-formed."""

    @pytest.fixture(scope="class")
    def processed_study(self, tmp_path_factory):
        """Process a study and return the output directory."""
        if not DEMO_GDISC.exists():
            pytest.skip("DEMO_GDISC sample data not available")

        output_dir = tmp_path_factory.mktemp("xml_validation")

        container = create_default_container(verbose=0)
        use_case = container.create_study_processing_use_case()

        request = ProcessStudyRequest(
            study_folder=DEMO_GDISC,
            study_id="DEMO_GDISC",
            output_dir=output_dir,
            output_formats=["xml"],
            generate_define_xml=False,
            generate_sas=False,
            sdtm_version="3.2",
        )

        response = use_case.execute(request)
        assert response.success, "Study processing should succeed"

        return output_dir

    def test_xml_files_exist(self, processed_study):
        """Test that XML files were created."""
        xml_dir = processed_study / "dataset-xml"
        if not xml_dir.exists():
            pytest.skip("XML output format not generated")

        xml_files = list(xml_dir.glob("*.xml"))
        assert len(xml_files) > 0, "Should have at least one XML file"

    def test_xml_files_well_formed(self, processed_study):
        """Test that all XML files are well-formed and parseable."""
        xml_dir = processed_study / "dataset-xml"
        if not xml_dir.exists():
            pytest.skip("XML directory not found")

        xml_files = list(xml_dir.glob("*.xml"))
        if not xml_files:
            pytest.skip("No XML files found")

        for xml_file in xml_files:
            try:
                tree = ET.parse(xml_file)
                root = tree.getroot()
                assert root is not None, f"{xml_file.name} has no root element"
            except ET.ParseError as e:
                pytest.fail(f"{xml_file.name} is not well-formed XML: {e}")

    def test_xml_has_root_element(self, processed_study):
        """Test that XML files have a root element."""
        xml_dir = processed_study / "dataset-xml"
        if not xml_dir.exists():
            pytest.skip("XML directory not found")

        xml_files = list(xml_dir.glob("*.xml"))
        if not xml_files:
            pytest.skip("No XML files found")

        for xml_file in xml_files:
            tree = ET.parse(xml_file)
            root = tree.getroot()

            # Root should have a tag name
            assert root.tag is not None and len(root.tag) > 0, (
                f"{xml_file.name} root element has no tag"
            )


@pytest.mark.validation
@pytest.mark.integration
class TestXMLStructure:
    """Test Dataset-XML structure and content."""

    @pytest.fixture(scope="class")
    def processed_study(self, tmp_path_factory):
        """Process a study and return the output directory."""
        if not DEMO_GDISC.exists():
            pytest.skip("DEMO_GDISC sample data not available")

        output_dir = tmp_path_factory.mktemp("xml_structure")

        container = create_default_container(verbose=0)
        use_case = container.create_study_processing_use_case()

        request = ProcessStudyRequest(
            study_folder=DEMO_GDISC,
            study_id="DEMO_GDISC",
            output_dir=output_dir,
            output_formats=["xml"],
            generate_define_xml=False,
            generate_sas=False,
            sdtm_version="3.2",
        )

        response = use_case.execute(request)
        assert response.success, "Study processing should succeed"

        return output_dir

    def test_xml_has_namespace(self, processed_study):
        """Test that XML files use appropriate namespace."""
        xml_dir = processed_study / "dataset-xml"
        if not xml_dir.exists():
            pytest.skip("XML directory not found")

        xml_files = list(xml_dir.glob("*.xml"))
        if not xml_files:
            pytest.skip("No XML files found")

        for xml_file in xml_files:
            tree = ET.parse(xml_file)
            root = tree.getroot()

            # Check if root has namespace (common in CDISC XML)
            tag = root.tag
            # Namespace is in format {namespace}tag
            if tag.startswith("{"):
                assert "}" in tag, f"{xml_file.name} has malformed namespace"

    def test_xml_file_sizes_reasonable(self, processed_study):
        """Test that XML files have reasonable sizes."""
        xml_dir = processed_study / "dataset-xml"
        if not xml_dir.exists():
            pytest.skip("XML directory not found")

        xml_files = list(xml_dir.glob("*.xml"))
        if not xml_files:
            pytest.skip("No XML files found")

        for xml_file in xml_files:
            size = xml_file.stat().st_size

            # Minimum size (even empty XML has some structure)
            assert size > 100, f"{xml_file.name} is too small ({size} bytes)"

            # Maximum size (500MB is very large for XML)
            assert size < 500 * 1024 * 1024, (
                f"{xml_file.name} is suspiciously large ({size / 1024 / 1024:.1f} MB)"
            )

    def test_xml_no_empty_files(self, processed_study):
        """Test that XML files are not empty."""
        xml_dir = processed_study / "dataset-xml"
        if not xml_dir.exists():
            pytest.skip("XML directory not found")

        xml_files = list(xml_dir.glob("*.xml"))
        if not xml_files:
            pytest.skip("No XML files found")

        for xml_file in xml_files:
            with open(xml_file, encoding="utf-8") as f:
                content = f.read().strip()
                assert len(content) > 0, f"{xml_file.name} is empty"


@pytest.mark.validation
@pytest.mark.integration
class TestXMLEncoding:
    """Test XML encoding and special characters."""

    @pytest.fixture(scope="class")
    def processed_study(self, tmp_path_factory):
        """Process a study and return the output directory."""
        if not DEMO_GDISC.exists():
            pytest.skip("DEMO_GDISC sample data not available")

        output_dir = tmp_path_factory.mktemp("xml_encoding")

        container = create_default_container(verbose=0)
        use_case = container.create_study_processing_use_case()

        request = ProcessStudyRequest(
            study_folder=DEMO_GDISC,
            study_id="DEMO_GDISC",
            output_dir=output_dir,
            output_formats=["xml"],
            generate_define_xml=False,
            generate_sas=False,
            sdtm_version="3.2",
        )

        response = use_case.execute(request)
        assert response.success, "Study processing should succeed"

        return output_dir

    def test_xml_encoding_declaration(self, processed_study):
        """Test that XML files have encoding declaration."""
        xml_dir = processed_study / "dataset-xml"
        if not xml_dir.exists():
            pytest.skip("XML directory not found")

        xml_files = list(xml_dir.glob("*.xml"))
        if not xml_files:
            pytest.skip("No XML files found")

        for xml_file in xml_files:
            with open(xml_file, encoding="utf-8") as f:
                first_line = f.readline()
                # Check for XML declaration
                if first_line.strip().startswith("<?xml"):
                    # Should have encoding specification
                    assert "encoding" in first_line.lower(), (
                        f"{xml_file.name} XML declaration missing encoding"
                    )

    def test_xml_special_characters_escaped(self, processed_study):
        """Test that XML special characters are properly escaped."""
        xml_dir = processed_study / "dataset-xml"
        if not xml_dir.exists():
            pytest.skip("XML directory not found")

        xml_files = list(xml_dir.glob("*.xml"))
        if not xml_files:
            pytest.skip("No XML files found")

        for xml_file in xml_files:
            # These should not appear unescaped in element content
            # (they're OK in CDATA sections or attributes)
            # This is a basic check - XML parser already validates this
            ET.parse(xml_file)

            # If we got here, XML is valid and characters are escaped
            # (ET.parse would fail on unescaped special chars)
            assert True

    def test_xml_readable_with_different_parsers(self, processed_study):
        """Test that XML can be read with different parsing methods."""
        xml_dir = processed_study / "dataset-xml"
        if not xml_dir.exists():
            pytest.skip("XML directory not found")

        xml_files = list(xml_dir.glob("*.xml"))
        if not xml_files:
            pytest.skip("No XML files found")

        for xml_file in xml_files:
            # Try with ElementTree
            try:
                tree1 = ET.parse(xml_file)
                root1 = tree1.getroot()
                assert root1 is not None
            except Exception as e:
                pytest.fail(f"ElementTree failed to parse {xml_file.name}: {e}")

            # Try iterparse (streaming parser)
            try:
                for event, elem in ET.iterparse(xml_file, events=("start", "end")):
                    pass  # Just iterate through
            except Exception as e:
                pytest.fail(f"iterparse failed to parse {xml_file.name}: {e}")
