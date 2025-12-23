"""Custom exceptions for infrastructure layer."""


class TranspilerInfrastructureError(Exception):
    """Base exception for infrastructure layer errors."""


class DataSourceError(TranspilerInfrastructureError):
    """Base exception for data source errors."""


class DataSourceNotFoundError(DataSourceError):
    """Raised when a data source file is not found."""


class DataParseError(DataSourceError):
    """Raised when data cannot be parsed."""


class DataValidationError(DataSourceError):
    """Raised when data fails validation."""
