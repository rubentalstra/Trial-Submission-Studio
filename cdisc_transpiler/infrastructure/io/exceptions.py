class TranspilerInfrastructureError(Exception):
    pass


class DataSourceError(TranspilerInfrastructureError):
    pass


class DataSourceNotFoundError(DataSourceError):
    pass


class DataParseError(DataSourceError):
    pass


class DataValidationError(DataSourceError):
    pass
