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

from pathlib import Path
from typing import Any

from rich.console import Console

from ..application.ports import FileGeneratorPort, LoggerPort, StudyDataRepositoryPort
from .io import CSVReader, FileGenerator
from .logging import ConsoleLogger, NullLogger
from .repositories import StudyDataRepository


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
        
        Returns:
            FileGeneratorPort implementation (FileGenerator)
            
        Example:
            >>> generator = container.create_file_generator()
            >>> result = generator.generate(request)
        """
        if self._file_generator_instance is None:
            self._file_generator_instance = FileGenerator()
        return self._file_generator_instance
    
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
    
    # Application Use Cases
    
    def create_study_processing_use_case(self):
        """Create a new study processing use case instance (transient).
        
        Note: Currently imports at runtime due to circular import issues.
        Returns a new instance each time (transient pattern).
        
        Returns:
            StudyProcessingUseCase instance with injected dependencies
            
        Example:
            >>> use_case = container.create_study_processing_use_case()
            >>> response = use_case.execute(request)
        """
        # Import here to avoid circular import at module level
        from ..application.study_processing_use_case import StudyProcessingUseCase
        
        logger = self.create_logger()
        return StudyProcessingUseCase(logger=logger)
    
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
        
        return DomainProcessingUseCase(
            logger=logger,
            study_data_repo=study_data_repo,
            file_generator=file_generator,
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
