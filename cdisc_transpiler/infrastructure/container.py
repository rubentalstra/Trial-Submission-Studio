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

from rich.console import Console

from ..application.ports import (
    DefineXmlGeneratorPort,
    FileGeneratorPort,
    LoggerPort,
    MappingPort,
    OutputPreparationPort,
    DomainDefinitionPort,
    StudyDataRepositoryPort,
)
from .io import (
    CSVReader,
    DatasetXMLWriter,
    DefineXmlGenerator,
    FileGenerator,
    OutputPreparer,
    SASWriter,
    XPTWriter,
)
from .logging import ConsoleLogger, NullLogger
from .repositories import DomainDefinitionRepository, StudyDataRepository
from .services.mapping_service_adapter import MappingServiceAdapter


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
        _study_data_repo_instance: Cached study data repository instance (singleton)

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
        self._study_data_repo_instance: StudyDataRepositoryPort | None = None
        self._define_xml_generator_instance: DefineXmlGeneratorPort | None = None
        self._output_preparer_instance: OutputPreparationPort | None = None
        self._xpt_writer_instance: XPTWriter | None = None
        self._domain_definition_repo_instance: DomainDefinitionPort | None = None
        self._mapping_service_instance: MappingPort | None = None

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

    def create_domain_definition_repository(self) -> DomainDefinitionPort:
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
        if self._study_data_repo_instance is None:
            csv_reader = self.create_csv_reader()
            self._study_data_repo_instance = StudyDataRepository(csv_reader=csv_reader)
        return self._study_data_repo_instance

    def create_define_xml_generator(self) -> DefineXmlGeneratorPort:
        """Create or return cached Define-XML generator instance (singleton).

        Returns:
            DefineXmlGeneratorPort implementation (DefineXmlGenerator)

        Example:
            >>> generator = container.create_define_xml_generator()
            >>> generator.generate(datasets, Path("define.xml"), sdtm_version="3.4", context="Submission")
        """
        if self._define_xml_generator_instance is None:
            self._define_xml_generator_instance = DefineXmlGenerator()
        return self._define_xml_generator_instance

    def create_output_preparer(self) -> OutputPreparationPort:
        """Create or return cached output preparer instance (singleton)."""
        if self._output_preparer_instance is None:
            self._output_preparer_instance = OutputPreparer()
        return self._output_preparer_instance

    def create_mapping_service(self) -> MappingPort:
        """Create or return cached mapping service instance (singleton)."""
        if self._mapping_service_instance is None:
            self._mapping_service_instance = MappingServiceAdapter()
        return self._mapping_service_instance

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
        from ..services import DomainDiscoveryService

        logger = self.create_logger()
        study_data_repo = self.create_study_data_repository()
        file_generator = self.create_file_generator()
        domain_processing_use_case = self.create_domain_processing_use_case()
        domain_definition_repo = self.create_domain_definition_repository()
        discovery_service = DomainDiscoveryService(logger=logger)
        define_xml_generator = self.create_define_xml_generator()
        output_preparer = self.create_output_preparer()

        return StudyProcessingUseCase(
            logger=logger,
            study_data_repo=study_data_repo,
            domain_processing_use_case=domain_processing_use_case,
            discovery_service=discovery_service,
            file_generator=file_generator,
            define_xml_generator=define_xml_generator,
            output_preparer=output_preparer,
            domain_definitions=domain_definition_repo,
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
        study_data_repo = self.create_study_data_repository()
        file_generator = self.create_file_generator()
        output_preparer = self.create_output_preparer()
        mapping_service = self.create_mapping_service()
        domain_definition_repo = self.create_domain_definition_repository()
        xpt_writer = self.create_xpt_writer()

        return DomainProcessingUseCase(
            logger=logger,
            study_data_repo=study_data_repo,
            file_generator=file_generator,
            mapping_service=mapping_service,
            output_preparer=output_preparer,
            domain_definitions=domain_definition_repo,
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
        self._study_data_repo_instance = None
        self._output_preparer_instance = None
        self._xpt_writer_instance = None
        self._domain_definition_repo_instance = None
        self._define_xml_generator_instance = None
        self._mapping_service_instance = None

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

    def override_study_data_repository(self, repo: StudyDataRepositoryPort) -> None:
        """Override the study data repository instance (for testing).

        Args:
            repo: Custom study data repository implementation

        Example:
            >>> mock_repo = MockStudyDataRepository()
            >>> container.override_study_data_repository(mock_repo)
        """
        self._study_data_repo_instance = repo

    def override_define_xml_generator(self, generator: DefineXmlGeneratorPort) -> None:
        """Override the Define-XML generator instance (for testing).

        Args:
            generator: Custom Define-XML generator implementation

        Example:
            >>> mock_generator = MockDefineXmlGenerator()
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
