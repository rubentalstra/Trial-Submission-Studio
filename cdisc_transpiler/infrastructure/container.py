"""Dependency injection container for the CDISC Transpiler application.

This module provides a simple dependency injection container that creates
and wires up all application dependencies. It follows the Dependency Injection
pattern to enable testability and flexibility.

The container provides factory methods for creating:
- Infrastructure components (CSV reader, file generator, logger)
- Application use cases
- Domain services

Example:
    >>> container = DependencyContainer(verbose=1)
    >>> logger = container.create_logger()
    >>> use_case = container.create_study_processing_use_case()
    >>> response = use_case.execute(request)
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from rich.console import Console

from ..application.ports import (
    DomainFrameBuilderPort,
    DefineXMLGeneratorPort,
    FileGeneratorPort,
    LoggerPort,
    MappingPort,
    OutputPreparerPort,
    SuppqualPort,
    TerminologyPort,
    DomainDefinitionRepositoryPort,
    StudyDataRepositoryPort,
)
from .io import (
    CSVReader,
    DatasetXMLWriter,
    DefineXMLGenerator,
    FileGenerator,
    OutputPreparer,
    SASWriter,
    XPTWriter,
)
from .logging import ConsoleLogger, NullLogger
from .repositories import DomainDefinitionRepository, StudyDataRepository
from .services.mapping_service_adapter import MappingServiceAdapter

if TYPE_CHECKING:
    from ..domain.services import RelrecService, SynthesisService


class DependencyContainer:
    """Dependency injection container for wiring up application components.

    This container provides factory methods for creating all application
    dependencies. It supports configuration injection and makes it easy
    to swap implementations for testing.

    Attributes:
        verbose: Verbosity level (0=normal, 1=verbose, 2=debug)
        console: Rich console for output (optional)
        _logger_instance: Cached logger instance (singleton)
        _file_generator_instance: Cached file generator instance (singleton)
        _study_data_repository_instance: Cached study data repository instance (singleton)

    Example:
        >>> # Create container with configuration
        >>> container = DependencyContainer(verbose=1)
        >>>
        >>> # Get singleton instances
        >>> logger = container.create_logger()
        >>> file_gen = container.create_file_generator()
        >>> study_repo = container.create_study_data_repository()
        >>>
        >>> # Create use cases (transient)
        >>> use_case = container.create_study_processing_use_case()
    """

    def __init__(
        self,
        verbose: int = 0,
        console: Console | None = None,
        use_null_logger: bool = False,
    ):
        """Initialize the dependency container.

        Args:
            verbose: Verbosity level (0=normal, 1=verbose, 2=debug)
            console: Rich console for output (None = create new)
            use_null_logger: Use NullLogger instead of ConsoleLogger (for testing)
        """
        self.verbose = verbose
        self.console = console or Console()
        self.use_null_logger = use_null_logger

        # Singleton instances
        self._logger_instance: LoggerPort | None = None
        self._file_generator_instance: FileGeneratorPort | None = None
        self._csv_reader_instance: CSVReader | None = None
        self._study_data_repository_instance: StudyDataRepositoryPort | None = None
        self._define_xml_generator_instance: DefineXMLGeneratorPort | None = None
        self._output_preparer_instance: OutputPreparerPort | None = None
        self._xpt_writer_instance: XPTWriter | None = None
        self._domain_definition_repo_instance: DomainDefinitionRepositoryPort | None = (
            None
        )
        self._mapping_service_instance: MappingPort | None = None
        self._domain_frame_builder_instance: DomainFrameBuilderPort | None = None
        self._suppqual_service_instance: SuppqualPort | None = None
        self._terminology_service_instance: TerminologyPort | None = None
        self._synthesis_service_instance: "SynthesisService | None" = None
        self._relrec_service_instance: "RelrecService | None" = None

    # Infrastructure Components

    def create_logger(self) -> LoggerPort:
        """Create or return cached logger instance (singleton).

        Returns:
            LoggerPort implementation (ConsoleLogger or NullLogger)

        Example:
            >>> logger = container.create_logger()
            >>> logger.info("Processing started")
        """
        if self._logger_instance is None:
            if self.use_null_logger:
                self._logger_instance = NullLogger()
            else:
                self._logger_instance = ConsoleLogger(
                    console=self.console,
                    verbosity=self.verbose,
                )
        return self._logger_instance

    def create_csv_reader(self) -> CSVReader:
        """Create or return cached CSV reader instance (singleton).

        Returns:
            CSVReader instance for reading CSV files

        Example:
            >>> reader = container.create_csv_reader()
            >>> df = reader.read(Path("data.csv"))
        """
        if self._csv_reader_instance is None:
            self._csv_reader_instance = CSVReader()
        return self._csv_reader_instance

    def create_file_generator(self) -> FileGeneratorPort:
        """Create or return cached file generator instance (singleton).

        The file generator is configured with writer adapters for XPT,
        Dataset-XML, and SAS output formats.

        Returns:
            FileGeneratorPort implementation (FileGenerator)

        Example:
            >>> generator = container.create_file_generator()
            >>> result = generator.generate(request)
        """
        if self._file_generator_instance is None:
            # Create writer adapters
            xpt_writer = self.create_xpt_writer()
            xml_writer = DatasetXMLWriter()
            sas_writer = SASWriter()

            # Create file generator with injected writers
            self._file_generator_instance = FileGenerator(
                xpt_writer=xpt_writer,
                xml_writer=xml_writer,
                sas_writer=sas_writer,
            )
        return self._file_generator_instance

    def create_xpt_writer(self) -> XPTWriter:
        """Create or return cached XPT writer instance (singleton)."""
        if self._xpt_writer_instance is None:
            self._xpt_writer_instance = XPTWriter()
        return self._xpt_writer_instance

    def create_domain_definition_repository(self) -> DomainDefinitionRepositoryPort:
        """Create or return cached SDTM domain definition repository (singleton)."""
        if self._domain_definition_repo_instance is None:
            self._domain_definition_repo_instance = DomainDefinitionRepository()
        return self._domain_definition_repo_instance

    def create_study_data_repository(self) -> StudyDataRepositoryPort:
        """Create or return cached study data repository instance (singleton).

        Returns:
            StudyDataRepositoryPort implementation (StudyDataRepository)

        Example:
            >>> repo = container.create_study_data_repository()
            >>> df = repo.read_dataset(Path("data.csv"))
        """
        if self._study_data_repository_instance is None:
            csv_reader = self.create_csv_reader()
            self._study_data_repository_instance = StudyDataRepository(
                csv_reader=csv_reader
            )
        return self._study_data_repository_instance

    def create_define_xml_generator(self) -> DefineXMLGeneratorPort:
        """Create or return cached Define-XML generator instance (singleton).

        Returns:
            DefineXMLGeneratorPort implementation (DefineXMLGenerator)

        Example:
            >>> generator = container.create_define_xml_generator()
            >>> # Prefer canonical defaults:
            >>> # sdtm_version=SDTMVersions.DEFAULT_VERSION
            >>> # context=SDTMVersions.DEFINE_CONTEXT_SUBMISSION
            >>> generator.generate(datasets, Path("define.xml"), sdtm_version="3.4", context="Submission")
        """
        if self._define_xml_generator_instance is None:
            self._define_xml_generator_instance = DefineXMLGenerator()
        return self._define_xml_generator_instance

    def create_output_preparer(self) -> OutputPreparerPort:
        """Create or return cached output preparer instance (singleton)."""
        if self._output_preparer_instance is None:
            self._output_preparer_instance = OutputPreparer()
        return self._output_preparer_instance

    def create_mapping_service(self) -> MappingPort:
        """Create or return cached mapping service instance (singleton)."""
        if self._mapping_service_instance is None:
            self._mapping_service_instance = MappingServiceAdapter(
                domain_definition_repository=self.create_domain_definition_repository()
            )
        return self._mapping_service_instance

    def create_domain_frame_builder(self) -> DomainFrameBuilderPort:
        """Create or return cached domain frame builder adapter (singleton)."""
        if self._domain_frame_builder_instance is None:
            from .services.domain_frame_builder_adapter import DomainFrameBuilderAdapter

            self._domain_frame_builder_instance = DomainFrameBuilderAdapter()
        return self._domain_frame_builder_instance

    def create_suppqual_service(self) -> SuppqualPort:
        """Create or return cached SUPPQUAL adapter (singleton)."""
        if self._suppqual_service_instance is None:
            from .services.suppqual_service_adapter import SuppqualServiceAdapter

            self._suppqual_service_instance = SuppqualServiceAdapter()
        return self._suppqual_service_instance

    def create_terminology_service(self) -> TerminologyPort:
        """Create or return cached terminology service adapter (singleton)."""
        if self._terminology_service_instance is None:
            from .services.terminology_service_adapter import TerminologyServiceAdapter

            self._terminology_service_instance = TerminologyServiceAdapter()
        return self._terminology_service_instance

    def create_synthesis_service(self) -> "SynthesisService":
        """Create or return cached synthesis service instance (singleton)."""
        if self._synthesis_service_instance is None:
            from ..domain.services import SynthesisService

            self._synthesis_service_instance = SynthesisService(
                domain_resolver=self.create_domain_definition_repository().get_domain
            )
        return self._synthesis_service_instance

    def create_relrec_service(self) -> "RelrecService":
        """Create or return cached RELREC service instance (singleton)."""
        if self._relrec_service_instance is None:
            from ..domain.services import RelrecService

            self._relrec_service_instance = RelrecService()
        return self._relrec_service_instance

    # Application Use Cases

    def create_study_processing_use_case(self):
        """Create a new study processing use case instance (transient).

        Returns a new instance each time with all dependencies wired.

        Returns:
            StudyProcessingUseCase instance with injected dependencies

        Example:
            >>> use_case = container.create_study_processing_use_case()
            >>> response = use_case.execute(request)
        """
        # Import here to avoid circular import at module level
        from ..application.study_processing_use_case import StudyProcessingUseCase
        from .services.domain_discovery_service_adapter import (
            DomainDiscoveryServiceAdapter,
        )

        logger = self.create_logger()
        study_data_repository = self.create_study_data_repository()
        file_generator = self.create_file_generator()
        domain_processing_use_case = self.create_domain_processing_use_case()
        domain_definition_repo = self.create_domain_definition_repository()
        domain_discovery_service = DomainDiscoveryServiceAdapter(logger=logger)
        domain_frame_builder = self.create_domain_frame_builder()
        synthesis_service = self.create_synthesis_service()
        relrec_service = self.create_relrec_service()
        define_xml_generator = self.create_define_xml_generator()
        output_preparer = self.create_output_preparer()

        return StudyProcessingUseCase(
            logger=logger,
            study_data_repository=study_data_repository,
            domain_processing_use_case=domain_processing_use_case,
            domain_discovery_service=domain_discovery_service,
            domain_frame_builder=domain_frame_builder,
            synthesis_service=synthesis_service,
            relrec_service=relrec_service,
            file_generator=file_generator,
            define_xml_generator=define_xml_generator,
            output_preparer=output_preparer,
            domain_definition_repository=domain_definition_repo,
        )

    def create_domain_processing_use_case(self):
        """Create a new domain processing use case instance (transient).

        Returns a new instance each time with all dependencies wired.

        Returns:
            DomainProcessingUseCase instance with injected dependencies

        Example:
            >>> use_case = container.create_domain_processing_use_case()
            >>> response = use_case.execute(request)
        """
        # Import here to avoid circular import at module level
        from ..application.domain_processing_use_case import DomainProcessingUseCase

        logger = self.create_logger()
        study_data_repository = self.create_study_data_repository()
        file_generator = self.create_file_generator()
        output_preparer = self.create_output_preparer()
        mapping_service = self.create_mapping_service()
        domain_definition_repo = self.create_domain_definition_repository()
        domain_frame_builder = self.create_domain_frame_builder()
        suppqual_service = self.create_suppqual_service()
        terminology_service = self.create_terminology_service()
        xpt_writer = self.create_xpt_writer()

        return DomainProcessingUseCase(
            logger=logger,
            study_data_repository=study_data_repository,
            file_generator=file_generator,
            mapping_service=mapping_service,
            output_preparer=output_preparer,
            domain_frame_builder=domain_frame_builder,
            suppqual_service=suppqual_service,
            terminology_service=terminology_service,
            domain_definition_repository=domain_definition_repo,
            xpt_writer=xpt_writer,
        )

    # Helper Methods

    def reset_singletons(self) -> None:
        """Reset all singleton instances.

        Useful for testing when you need to create fresh instances.

        Example:
            >>> container.reset_singletons()
            >>> new_logger = container.create_logger()  # Fresh instance
        """
        self._logger_instance = None
        self._file_generator_instance = None
        self._csv_reader_instance = None
        self._study_data_repository_instance = None
        self._output_preparer_instance = None
        self._xpt_writer_instance = None
        self._domain_definition_repo_instance = None
        self._define_xml_generator_instance = None
        self._mapping_service_instance = None
        self._domain_frame_builder_instance = None
        self._suppqual_service_instance = None
        self._terminology_service_instance = None
        self._synthesis_service_instance = None
        self._relrec_service_instance = None

    def override_logger(self, logger: LoggerPort) -> None:
        """Override the logger instance (for testing).

        Args:
            logger: Custom logger implementation

        Example:
            >>> mock_logger = MockLogger()
            >>> container.override_logger(mock_logger)
        """
        self._logger_instance = logger

    def override_file_generator(self, file_generator: FileGeneratorPort) -> None:
        """Override the file generator instance (for testing).

        Args:
            file_generator: Custom file generator implementation

        Example:
            >>> mock_generator = MockFileGenerator()
            >>> container.override_file_generator(mock_generator)
        """
        self._file_generator_instance = file_generator

    def override_study_data_repository(
        self, study_data_repository: StudyDataRepositoryPort
    ) -> None:
        """Override the study data repository instance (for testing).

        Args:
            repo: Custom study data repository implementation

        Example:
            >>> mock_repo = MockStudyDataRepository()
            >>> container.override_study_data_repository(mock_repo)
        """
        self._study_data_repository_instance = study_data_repository

    def override_define_xml_generator(self, generator: DefineXMLGeneratorPort) -> None:
        """Override the Define-XML generator instance (for testing).

        Args:
            generator: Custom Define-XML generator implementation

        Example:
            >>> mock_generator = MockDefineXMLGenerator()
            >>> container.override_define_xml_generator(mock_generator)
        """
        self._define_xml_generator_instance = generator


# Convenience function for creating a pre-configured container
def create_default_container(verbose: int = 0) -> DependencyContainer:
    """Create a dependency container with default configuration.

    This is a convenience function for creating a container with
    standard settings.

    Args:
        verbose: Verbosity level (0=normal, 1=verbose, 2=debug)

    Returns:
        Configured DependencyContainer instance

    Example:
        >>> container = create_default_container(verbose=1)
        >>> logger = container.create_logger()
    """
    return DependencyContainer(verbose=verbose)
