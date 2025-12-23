"""Domain services.

Business logic services that operate on domain entities.
"""

from .domain_frame_builder import (
    DomainFrameBuilder,
    DomainFrameBuildError,
    build_domain_dataframe,
)
from .domain_processors import (
    BaseDomainProcessor,
    DefaultDomainProcessor,
    DomainProcessorRegistry,
    get_domain_processor,
)
from .relrec_service import (
    RelrecService,
)
from .relspec_service import (
    RelspecService,
)
from .relsub_service import (
    RelsubService,
)
from .suppqual_service import (
    build_suppqual,
    extract_used_columns,
    finalize_suppqual,
    sanitize_qnam,
)

__all__ = [
    # Domain frame builder
    "DomainFrameBuildError",
    "DomainFrameBuilder",
    "build_domain_dataframe",
    # Domain processors
    "BaseDomainProcessor",
    "DefaultDomainProcessor",
    "DomainProcessorRegistry",
    "get_domain_processor",
    # SUPPQUAL service
    "build_suppqual",
    "extract_used_columns",
    "finalize_suppqual",
    "sanitize_qnam",
    # RELREC service
    "RelrecService",
    # RELSUB service
    "RelsubService",
    # RELSPEC service
    "RelspecService",
]
