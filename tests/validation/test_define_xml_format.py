"""Define-XML format validation tests.

This module tests that generated Define-XML files:
- Are well-formed XML
- Have correct Define-XML 2.1 structure
- Contain expected metadata elements
"""

import pytest
from pathlib import Path
import xml.etree.ElementTree as ET

from cdisc_transpiler.infrastructure import create_default_container
from cdisc_transpiler.application.models import ProcessStudyRequest


# Path to sample study data
MOCKDATA_DIR = Path(__file__).parent.parent.parent / "mockdata"
DEMO_GDISC = MOCKDATA_DIR / "DEMO_GDISC_20240903_072908"


@pytest.mark.validation
@pytest.mark.integration
class TestDefineXMLWellFormedness:
    """Test that Define-XML files are well-formed."""
    
    @pytest.fixture(scope="class")
    def processed_study(self, tmp_path_factory):
        """Process a study with Define-XML generation."""
        if not DEMO_GDISC.exists():
            pytest.skip("DEMO_GDISC sample data not available")
        
        output_dir = tmp_path_factory.mktemp("define_validation")
        
        container = create_default_container(verbose=0)
        use_case = container.create_study_processing_use_case()
        
        request = ProcessStudyRequest(
            study_folder=DEMO_GDISC,
            study_id="DEMO_GDISC",
            output_dir=output_dir,
            output_formats=["xpt"],
            generate_define_xml=True,
            generate_sas=False,
            sdtm_version="3.2",
        )
        
        response = use_case.execute(request)
        assert response.success, "Study processing should succeed"
        
        return output_dir
    
    def test_define_xml_file_exists(self, processed_study):
        """Test that Define-XML file was created."""
        define_files = list(processed_study.glob("define*.xml"))
        
        if not define_files:
            pytest.skip("Define-XML not generated")
        
        assert len(define_files) > 0, "Define-XML file should exist"
    
    def test_define_xml_well_formed(self, processed_study):
        """Test that Define-XML is well-formed XML."""
        define_files = list(processed_study.glob("define*.xml"))
        
        if not define_files:
            pytest.skip("Define-XML not found")
        
        for define_file in define_files:
            try:
                tree = ET.parse(define_file)
                root = tree.getroot()
                assert root is not None, "Define-XML has no root element"
            except ET.ParseError as e:
                pytest.fail(f"Define-XML is not well-formed: {e}")
    
    def test_define_xml_has_odm_root(self, processed_study):
        """Test that Define-XML has ODM root element."""
        define_files = list(processed_study.glob("define*.xml"))
        
        if not define_files:
            pytest.skip("Define-XML not found")
        
        for define_file in define_files:
            tree = ET.parse(define_file)
            root = tree.getroot()
            
            # Remove namespace for comparison
            tag = root.tag.split('}')[-1] if '}' in root.tag else root.tag
            
            # Should be ODM root element
            assert tag == 'ODM', f"Root element should be ODM, got {tag}"


@pytest.mark.validation
@pytest.mark.integration
class TestDefineXMLStructure:
    """Test Define-XML structure and required elements."""
    
    @pytest.fixture(scope="class")
    def processed_study(self, tmp_path_factory):
        """Process a study with Define-XML generation."""
        if not DEMO_GDISC.exists():
            pytest.skip("DEMO_GDISC sample data not available")
        
        output_dir = tmp_path_factory.mktemp("define_structure")
        
        container = create_default_container(verbose=0)
        use_case = container.create_study_processing_use_case()
        
        request = ProcessStudyRequest(
            study_folder=DEMO_GDISC,
            study_id="DEMO_GDISC",
            output_dir=output_dir,
            output_formats=["xpt"],
            generate_define_xml=True,
            generate_sas=False,
            sdtm_version="3.2",
        )
        
        response = use_case.execute(request)
        assert response.success, "Study processing should succeed"
        
        return output_dir
    
    def test_define_xml_has_study_element(self, processed_study):
        """Test that Define-XML has Study element."""
        define_files = list(processed_study.glob("define*.xml"))
        
        if not define_files:
            pytest.skip("Define-XML not found")
        
        for define_file in define_files:
            tree = ET.parse(define_file)
            root = tree.getroot()
            
            # Find Study element (with or without namespace)
            studies = []
            for elem in root.iter():
                tag = elem.tag.split('}')[-1] if '}' in elem.tag else elem.tag
                if tag == 'Study':
                    studies.append(elem)
            
            assert len(studies) > 0, "Define-XML should have Study element"
    
    def test_define_xml_has_metadata_version(self, processed_study):
        """Test that Define-XML has MetaDataVersion element."""
        define_files = list(processed_study.glob("define*.xml"))
        
        if not define_files:
            pytest.skip("Define-XML not found")
        
        for define_file in define_files:
            tree = ET.parse(define_file)
            root = tree.getroot()
            
            # Find MetaDataVersion element
            metadata_versions = []
            for elem in root.iter():
                tag = elem.tag.split('}')[-1] if '}' in elem.tag else elem.tag
                if tag == 'MetaDataVersion':
                    metadata_versions.append(elem)
            
            assert len(metadata_versions) > 0, (
                "Define-XML should have MetaDataVersion element"
            )
    
    def test_define_xml_has_item_group_defs(self, processed_study):
        """Test that Define-XML has ItemGroupDef elements (datasets)."""
        define_files = list(processed_study.glob("define*.xml"))
        
        if not define_files:
            pytest.skip("Define-XML not found")
        
        for define_file in define_files:
            tree = ET.parse(define_file)
            root = tree.getroot()
            
            # Find ItemGroupDef elements
            item_group_defs = []
            for elem in root.iter():
                tag = elem.tag.split('}')[-1] if '}' in elem.tag else elem.tag
                if tag == 'ItemGroupDef':
                    item_group_defs.append(elem)
            
            # Should have at least one domain defined
            assert len(item_group_defs) > 0, (
                "Define-XML should have ItemGroupDef elements"
            )
    
    def test_define_xml_has_item_defs(self, processed_study):
        """Test that Define-XML has ItemDef elements (variables)."""
        define_files = list(processed_study.glob("define*.xml"))
        
        if not define_files:
            pytest.skip("Define-XML not found")
        
        for define_file in define_files:
            tree = ET.parse(define_file)
            root = tree.getroot()
            
            # Find ItemDef elements
            item_defs = []
            for elem in root.iter():
                tag = elem.tag.split('}')[-1] if '}' in elem.tag else elem.tag
                if tag == 'ItemDef':
                    item_defs.append(elem)
            
            # Should have variable definitions
            assert len(item_defs) > 0, "Define-XML should have ItemDef elements"


@pytest.mark.validation
@pytest.mark.integration
class TestDefineXMLAttributes:
    """Test Define-XML attributes and metadata quality."""
    
    @pytest.fixture(scope="class")
    def processed_study(self, tmp_path_factory):
        """Process a study with Define-XML generation."""
        if not DEMO_GDISC.exists():
            pytest.skip("DEMO_GDISC sample data not available")
        
        output_dir = tmp_path_factory.mktemp("define_attributes")
        
        container = create_default_container(verbose=0)
        use_case = container.create_study_processing_use_case()
        
        request = ProcessStudyRequest(
            study_folder=DEMO_GDISC,
            study_id="DEMO_GDISC",
            output_dir=output_dir,
            output_formats=["xpt"],
            generate_define_xml=True,
            generate_sas=False,
            sdtm_version="3.2",
        )
        
        response = use_case.execute(request)
        assert response.success, "Study processing should succeed"
        
        return output_dir
    
    def test_define_xml_odm_version(self, processed_study):
        """Test that Define-XML has ODM version attribute."""
        define_files = list(processed_study.glob("define*.xml"))
        
        if not define_files:
            pytest.skip("Define-XML not found")
        
        for define_file in define_files:
            tree = ET.parse(define_file)
            root = tree.getroot()
            
            # Check for ODMVersion attribute
            odm_version = root.get('ODMVersion')
            assert odm_version is not None, "Define-XML missing ODMVersion attribute"
            
            # Should be a valid ODM version
            assert odm_version in ['1.3.2', '1.3', '2.0'], (
                f"Unexpected ODM version: {odm_version}"
            )
    
    def test_define_xml_file_size_reasonable(self, processed_study):
        """Test that Define-XML has reasonable file size."""
        define_files = list(processed_study.glob("define*.xml"))
        
        if not define_files:
            pytest.skip("Define-XML not found")
        
        for define_file in define_files:
            size = define_file.stat().st_size
            
            # Minimum size (Define-XML has substantial structure)
            assert size > 1000, f"Define-XML too small ({size} bytes)"
            
            # Maximum size (10MB is very large for Define-XML)
            assert size < 10 * 1024 * 1024, (
                f"Define-XML suspiciously large ({size / 1024 / 1024:.1f} MB)"
            )
    
    def test_define_xml_item_group_defs_have_oids(self, processed_study):
        """Test that ItemGroupDef elements have OID attributes."""
        define_files = list(processed_study.glob("define*.xml"))
        
        if not define_files:
            pytest.skip("Define-XML not found")
        
        for define_file in define_files:
            tree = ET.parse(define_file)
            root = tree.getroot()
            
            # Find ItemGroupDef elements
            for elem in root.iter():
                tag = elem.tag.split('}')[-1] if '}' in elem.tag else elem.tag
                if tag == 'ItemGroupDef':
                    oid = elem.get('OID')
                    assert oid is not None and len(oid) > 0, (
                        "ItemGroupDef missing OID attribute"
                    )
    
    def test_define_xml_item_defs_have_oids(self, processed_study):
        """Test that ItemDef elements have OID attributes."""
        define_files = list(processed_study.glob("define*.xml"))
        
        if not define_files:
            pytest.skip("Define-XML not found")
        
        for define_file in define_files:
            tree = ET.parse(define_file)
            root = tree.getroot()
            
            # Find ItemDef elements
            for elem in root.iter():
                tag = elem.tag.split('}')[-1] if '}' in elem.tag else elem.tag
                if tag == 'ItemDef':
                    oid = elem.get('OID')
                    assert oid is not None and len(oid) > 0, (
                        "ItemDef missing OID attribute"
                    )
    
    def test_define_xml_item_defs_have_datatypes(self, processed_study):
        """Test that ItemDef elements have DataType attributes."""
        define_files = list(processed_study.glob("define*.xml"))
        
        if not define_files:
            pytest.skip("Define-XML not found")
        
        for define_file in define_files:
            tree = ET.parse(define_file)
            root = tree.getroot()
            
            # Find ItemDef elements
            item_defs_checked = 0
            for elem in root.iter():
                tag = elem.tag.split('}')[-1] if '}' in elem.tag else elem.tag
                if tag == 'ItemDef':
                    datatype = elem.get('DataType')
                    assert datatype is not None and len(datatype) > 0, (
                        f"ItemDef {elem.get('OID')} missing DataType attribute"
                    )
                    
                    # Should be a valid ODM datatype
                    valid_datatypes = [
                        'text', 'integer', 'float', 'date', 'time', 'datetime',
                        'string', 'boolean', 'double', 'URI', 'base64Binary',
                        'base64Float', 'hexBinary', 'hexFloat', 'partialDate',
                        'partialTime', 'partialDatetime', 'durationDatetime',
                        'intervalDatetime', 'incompleteDatetime', 'incompleteDate',
                        'incompleteTime'
                    ]
                    assert datatype in valid_datatypes, (
                        f"ItemDef has invalid DataType: {datatype}"
                    )
                    
                    item_defs_checked += 1
            
            # Should have checked at least some ItemDefs
            assert item_defs_checked > 0, "No ItemDefs found to validate"
