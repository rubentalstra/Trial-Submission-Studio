"""Performance benchmark tests for CDISC Transpiler.

This module contains performance benchmarks to track processing speed
and detect performance regressions. Tests use pytest-benchmark.
"""

import pytest
from pathlib import Path
import tempfile

from cdisc_transpiler.infrastructure import create_default_container
from cdisc_transpiler.application.models import ProcessStudyRequest


# Path to sample study data
MOCKDATA_DIR = Path(__file__).parent.parent.parent / "mockdata"
DEMO_CF = MOCKDATA_DIR / "DEMO_CF1234_NL_20250120_104838"
DEMO_GDISC = MOCKDATA_DIR / "DEMO_GDISC_20240903_072908"


@pytest.mark.benchmark
@pytest.mark.integration
class TestStudyProcessingPerformance:
    """Performance benchmarks for study processing workflows."""
    
    @pytest.fixture
    def small_study_folder(self):
        """Provide path to small study (DEMO_CF)."""
        if not DEMO_CF.exists():
            pytest.skip("DEMO_CF sample data not available")
        return DEMO_CF
    
    @pytest.fixture
    def large_study_folder(self):
        """Provide path to large study (DEMO_GDISC)."""
        if not DEMO_GDISC.exists():
            pytest.skip("DEMO_GDISC sample data not available")
        return DEMO_GDISC
    
    @pytest.fixture
    def container(self):
        """Create dependency container with null logger for testing."""
        return create_default_container(verbose=0)
    
    def test_benchmark_small_study_processing(self, benchmark, small_study_folder, container):
        """Benchmark processing of small study (DEMO_CF, ~11 domains, 59 records).
        
        This benchmark measures the complete study processing pipeline including:
        - File discovery
        - Domain processing
        - XPT generation
        
        Expected baseline: <5 seconds on typical hardware.
        """
        use_case = container.create_study_processing_use_case()
        
        def process_study():
            with tempfile.TemporaryDirectory() as tmp_dir:
                output_dir = Path(tmp_dir) / "output"
                
                request = ProcessStudyRequest(
                    study_folder=small_study_folder,
                    study_id="DEMO_CF",
                    output_dir=output_dir,
                    output_formats=["xpt"],
                    generate_define_xml=False,
                    generate_sas=False,
                )
                
                response = use_case.execute(request)
                return response
        
        # Run benchmark
        result = benchmark(process_study)
        
        # Verify result is valid
        assert result.success, "Study processing should succeed"
        assert len(result.domain_results) > 0, "Should process at least one domain"
    
    def test_benchmark_large_study_processing(self, benchmark, large_study_folder, container):
        """Benchmark processing of large study (DEMO_GDISC, ~18 domains, 260 records).
        
        This benchmark measures the complete study processing pipeline including:
        - File discovery (more files)
        - Domain processing (more domains)
        - Variant domain merging (LBCC, LBHM)
        - XPT generation (more files)
        
        Expected baseline: <20 seconds on typical hardware.
        """
        use_case = container.create_study_processing_use_case()
        
        def process_study():
            with tempfile.TemporaryDirectory() as tmp_dir:
                output_dir = Path(tmp_dir) / "output"
                
                request = ProcessStudyRequest(
                    study_folder=large_study_folder,
                    study_id="DEMO_GDISC",
                    output_dir=output_dir,
                    output_formats=["xpt"],
                    generate_define_xml=False,
                    generate_sas=False,
                )
                
                response = use_case.execute(request)
                return response
        
        # Run benchmark
        result = benchmark(process_study)
        
        # Verify result is valid
        assert result.success, "Study processing should succeed"
        assert len(result.domain_results) >= 10, "Should process at least 10 domains"


@pytest.mark.benchmark
@pytest.mark.integration
class TestDomainProcessingPerformance:
    """Performance benchmarks for individual domain processing."""
    
    @pytest.fixture
    def study_folder(self):
        """Provide path to sample study folder."""
        if not DEMO_GDISC.exists():
            pytest.skip("Sample data not available")
        return DEMO_GDISC
    
    @pytest.fixture
    def container(self):
        """Create dependency container."""
        return create_default_container(verbose=0)
    
    def test_benchmark_dm_domain_processing(self, benchmark, study_folder, container):
        """Benchmark DM (Demographics) domain processing.
        
        DM is typically the largest and most complex domain.
        Expected baseline: <2 seconds on typical hardware.
        """
        from cdisc_transpiler.application.models import ProcessDomainRequest
        
        use_case = container.create_domain_processing_use_case()
        
        def process_dm_domain():
            with tempfile.TemporaryDirectory() as tmp_dir:
                output_dir = Path(tmp_dir) / "output"
                
                request = ProcessDomainRequest(
                    study_folder=study_folder,
                    domain_code="DM",
                    output_dir=output_dir,
                    output_formats=["xpt"],
                    study_id="DEMO_GDISC",
                )
                
                response = use_case.execute(request)
                return response
        
        # Run benchmark
        result = benchmark(process_dm_domain)
        
        # Verify result is valid
        assert result.success, "Domain processing should succeed"
        assert result.domain_code == "DM"
    
    def test_benchmark_ae_domain_processing(self, benchmark, study_folder, container):
        """Benchmark AE (Adverse Events) domain processing.
        
        AE is a common findings domain with moderate complexity.
        Expected baseline: <1 second on typical hardware.
        """
        from cdisc_transpiler.application.models import ProcessDomainRequest
        
        use_case = container.create_domain_processing_use_case()
        
        def process_ae_domain():
            with tempfile.TemporaryDirectory() as tmp_dir:
                output_dir = Path(tmp_dir) / "output"
                
                request = ProcessDomainRequest(
                    study_folder=study_folder,
                    domain_code="AE",
                    output_dir=output_dir,
                    output_formats=["xpt"],
                    study_id="DEMO_GDISC",
                )
                
                response = use_case.execute(request)
                return response
        
        # Run benchmark
        result = benchmark(process_ae_domain)
        
        # Verify result is valid
        assert result.success, "Domain processing should succeed"
        assert result.domain_code == "AE"


@pytest.mark.benchmark
class TestTransformationPerformance:
    """Performance benchmarks for data transformations."""
    
    def test_benchmark_dataframe_operations(self, benchmark):
        """Benchmark basic pandas operations used in transformations.
        
        This tests common DataFrame operations used throughout the codebase.
        Expected baseline: <10ms for 1000 rows on typical hardware.
        """
        import pandas as pd
        
        # Create sample data
        data = {
            "STUDYID": ["DEMO"] * 1000,
            "USUBJID": [f"SUBJ{i:04d}" for i in range(1000)],
            "VISIT": [f"Visit {i % 10}" for i in range(1000)],
            "TESTCD": ["ALT"] * 500 + ["AST"] * 500,
            "ORRES": [str(i) for i in range(1000)],
            "ORRESU": ["U/L"] * 1000,
        }
        df = pd.DataFrame(data)
        
        def perform_operations():
            # Common operations: filter, sort, groupby
            result = df.copy()
            result = result[result["TESTCD"].isin(["ALT", "AST"])]
            result = result.sort_values(["USUBJID", "VISIT", "TESTCD"])
            result["SEQ"] = result.groupby("USUBJID").cumcount() + 1
            return result
        
        # Run benchmark
        result = benchmark(perform_operations)
        
        # Verify result
        assert not result.empty, "Should produce results"
        assert "SEQ" in result.columns, "Should add SEQ column"


# Benchmark configuration for pytest-benchmark
# These can be overridden via command line: --benchmark-min-rounds=10
def pytest_benchmark_generate_json(config, benchmarks, output_json):
    """Customize benchmark output.
    
    This hook is called after benchmarks run to customize the JSON output.
    """
    # Add metadata to benchmark results
    output_json["machine_info"]["project"] = "cdisc-transpiler"
    output_json["machine_info"]["purpose"] = "performance_regression_detection"
