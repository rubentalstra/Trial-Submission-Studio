from typing import TYPE_CHECKING

from rich.console import Console

from ..application.domain_processing_use_case import (
    DomainProcessingDependencies,
    DomainProcessingUseCase,
)
from ..application.study_processing_use_case import (
    StudyProcessingDependencies,
    StudyProcessingUseCase,
)
from ..domain.services.relrec_service import RelrecService
from ..domain.services.relspec_service import RelspecService
from ..domain.services.relsub_service import RelsubService
from .io.csv_reader import CSVReader
from .io.dataset_output import DatasetOutputAdapter
from .io.dataset_xml_writer import DatasetXMLWriter
from .io.define_xml_generator import DefineXMLGenerator
from .io.output_preparer import OutputPreparer
from .io.sas_writer import SASWriter
from .io.xpt_writer import XPTWriter
from .logging.console_logger import ConsoleLogger
from .logging.null_logger import NullLogger
from .repositories.ct_repository import get_default_ct_repository
from .repositories.domain_definition_repository import DomainDefinitionRepository
from .repositories.study_data_repository import StudyDataRepository
from .services.conformance_report_writer_adapter import ConformanceReportWriterAdapter
from .services.domain_discovery_adapter import DomainDiscoveryAdapter
from .services.domain_frame_builder_adapter import DomainFrameBuilderAdapter
from .services.mapping_service_adapter import MappingServiceAdapter
from .services.suppqual_service_adapter import SuppqualServiceAdapter

if TYPE_CHECKING:
    from ..application.ports.repositories import (
        CTRepositoryPort,
        DomainDefinitionRepositoryPort,
        StudyDataRepositoryPort,
    )
    from ..application.ports.services import (
        ConformanceReportWriterPort,
        DatasetOutputPort,
        DefineXMLGeneratorPort,
        DomainFrameBuilderPort,
        LoggerPort,
        MappingPort,
        OutputPreparerPort,
        SuppqualPort,
    )


class DependencyContainer:
    pass

    def __init__(
        self,
        verbose: int = 0,
        console: Console | None = None,
        use_null_logger: bool = False,
    ) -> None:
        super().__init__()
        self.verbose = verbose
        self.console = console or Console()
        self.use_null_logger = use_null_logger
        self._logger_instance: LoggerPort | None = None
        self._dataset_output_instance: DatasetOutputPort | None = None
        self._csv_reader_instance: CSVReader | None = None
        self._study_data_repository_instance: StudyDataRepositoryPort | None = None
        self._define_xml_generator_instance: DefineXMLGeneratorPort | None = None
        self._output_preparer_instance: OutputPreparerPort | None = None
        self._xpt_writer_instance: XPTWriter | None = None
        self._domain_definition_repository_instance: (
            DomainDefinitionRepositoryPort | None
        ) = None
        self._mapping_service_instance: MappingPort | None = None
        self._domain_frame_builder_instance: DomainFrameBuilderPort | None = None
        self._suppqual_service_instance: SuppqualPort | None = None
        self._ct_repository_instance: CTRepositoryPort | None = None
        self._relrec_service_instance: RelrecService | None = None
        self._relsub_service_instance: RelsubService | None = None
        self._relspec_service_instance: RelspecService | None = None
        self._conformance_report_writer_instance: ConformanceReportWriterPort | None = (
            None
        )

    def create_logger(self) -> LoggerPort:
        if self._logger_instance is None:
            if self.use_null_logger:
                self._logger_instance = NullLogger()
            else:
                self._logger_instance = ConsoleLogger(
                    console=self.console, verbosity=self.verbose
                )
        return self._logger_instance

    def create_csv_reader(self) -> CSVReader:
        if self._csv_reader_instance is None:
            self._csv_reader_instance = CSVReader()
        return self._csv_reader_instance

    def create_dataset_output(self) -> DatasetOutputPort:
        if self._dataset_output_instance is None:
            xpt_writer = self.create_xpt_writer()
            xml_writer = DatasetXMLWriter()
            sas_writer = SASWriter()
            self._dataset_output_instance = DatasetOutputAdapter(
                xpt_writer=xpt_writer, xml_writer=xml_writer, sas_writer=sas_writer
            )
        return self._dataset_output_instance

    def create_xpt_writer(self) -> XPTWriter:
        if self._xpt_writer_instance is None:
            self._xpt_writer_instance = XPTWriter()
        return self._xpt_writer_instance

    def create_domain_definition_repository(self) -> DomainDefinitionRepositoryPort:
        if self._domain_definition_repository_instance is None:
            self._domain_definition_repository_instance = DomainDefinitionRepository()
        return self._domain_definition_repository_instance

    def create_study_data_repository(self) -> StudyDataRepositoryPort:
        if self._study_data_repository_instance is None:
            csv_reader = self.create_csv_reader()
            self._study_data_repository_instance = StudyDataRepository(
                csv_reader=csv_reader
            )
        return self._study_data_repository_instance

    def create_define_xml_generator(self) -> DefineXMLGeneratorPort:
        if self._define_xml_generator_instance is None:
            self._define_xml_generator_instance = DefineXMLGenerator()
        return self._define_xml_generator_instance

    def create_output_preparer(self) -> OutputPreparerPort:
        if self._output_preparer_instance is None:
            self._output_preparer_instance = OutputPreparer()
        return self._output_preparer_instance

    def create_mapping_service(self) -> MappingPort:
        if self._mapping_service_instance is None:
            self._mapping_service_instance = MappingServiceAdapter(
                domain_definition_repository=self.create_domain_definition_repository()
            )
        return self._mapping_service_instance

    def create_domain_frame_builder(self) -> DomainFrameBuilderPort:
        if self._domain_frame_builder_instance is None:
            self._domain_frame_builder_instance = DomainFrameBuilderAdapter(
                ct_repository=self.create_ct_repository()
            )
        return self._domain_frame_builder_instance

    def create_suppqual_service(self) -> SuppqualPort:
        if self._suppqual_service_instance is None:
            self._suppqual_service_instance = SuppqualServiceAdapter()
        return self._suppqual_service_instance

    def create_ct_repository(self) -> CTRepositoryPort:
        if self._ct_repository_instance is None:
            self._ct_repository_instance = get_default_ct_repository()
        return self._ct_repository_instance

    def create_conformance_report_writer(self) -> ConformanceReportWriterPort:
        if self._conformance_report_writer_instance is None:
            self._conformance_report_writer_instance = ConformanceReportWriterAdapter()
        return self._conformance_report_writer_instance

    def create_relrec_service(self) -> RelrecService:
        if self._relrec_service_instance is None:
            self._relrec_service_instance = RelrecService()
        return self._relrec_service_instance

    def create_relsub_service(self) -> RelsubService:
        if self._relsub_service_instance is None:
            self._relsub_service_instance = RelsubService()
        return self._relsub_service_instance

    def create_relspec_service(self) -> RelspecService:
        if self._relspec_service_instance is None:
            self._relspec_service_instance = RelspecService()
        return self._relspec_service_instance

    def create_study_processing_use_case(self) -> StudyProcessingUseCase:
        logger = self.create_logger()
        study_data_repository = self.create_study_data_repository()
        dataset_output = self.create_dataset_output()
        domain_processing_use_case = self.create_domain_processing_use_case()
        domain_definition_repository = self.create_domain_definition_repository()
        domain_discovery_service = DomainDiscoveryAdapter(logger=logger)
        domain_frame_builder = self.create_domain_frame_builder()
        relrec_service = self.create_relrec_service()
        relsub_service = self.create_relsub_service()
        relspec_service = self.create_relspec_service()
        define_xml_generator = self.create_define_xml_generator()
        output_preparer = self.create_output_preparer()
        ct_repository = self.create_ct_repository()
        conformance_report_writer = self.create_conformance_report_writer()
        dependencies = StudyProcessingDependencies(
            logger=logger,
            study_data_repository=study_data_repository,
            domain_processing_use_case=domain_processing_use_case,
            domain_discovery_service=domain_discovery_service,
            domain_frame_builder=domain_frame_builder,
            relrec_service=relrec_service,
            relsub_service=relsub_service,
            relspec_service=relspec_service,
            domain_definition_repository=domain_definition_repository,
            dataset_output=dataset_output,
            define_xml_generator=define_xml_generator,
            output_preparer=output_preparer,
            ct_repository=ct_repository,
            conformance_report_writer=conformance_report_writer,
        )
        return StudyProcessingUseCase(dependencies)

    def create_domain_processing_use_case(self) -> DomainProcessingUseCase:
        logger = self.create_logger()
        study_data_repository = self.create_study_data_repository()
        dataset_output = self.create_dataset_output()
        mapping_service = self.create_mapping_service()
        domain_definition_repository = self.create_domain_definition_repository()
        domain_frame_builder = self.create_domain_frame_builder()
        suppqual_service = self.create_suppqual_service()
        ct_repository = self.create_ct_repository()
        dependencies = DomainProcessingDependencies(
            logger=logger,
            study_data_repository=study_data_repository,
            mapping_service=mapping_service,
            domain_frame_builder=domain_frame_builder,
            suppqual_service=suppqual_service,
            domain_definition_repository=domain_definition_repository,
            dataset_output=dataset_output,
            ct_repository=ct_repository,
        )
        return DomainProcessingUseCase(dependencies)

    def reset_singletons(self) -> None:
        self._logger_instance = None
        self._dataset_output_instance = None
        self._csv_reader_instance = None
        self._study_data_repository_instance = None
        self._output_preparer_instance = None
        self._xpt_writer_instance = None
        self._domain_definition_repository_instance = None
        self._define_xml_generator_instance = None
        self._mapping_service_instance = None
        self._domain_frame_builder_instance = None
        self._suppqual_service_instance = None
        self._ct_repository_instance = None
        self._relrec_service_instance = None
        self._relsub_service_instance = None
        self._relspec_service_instance = None
        self._conformance_report_writer_instance = None

    def override_logger(self, logger: LoggerPort) -> None:
        self._logger_instance = logger

    def override_dataset_output(self, dataset_output: DatasetOutputPort) -> None:
        self._dataset_output_instance = dataset_output

    def override_study_data_repository(
        self, study_data_repository: StudyDataRepositoryPort
    ) -> None:
        self._study_data_repository_instance = study_data_repository

    def override_define_xml_generator(self, generator: DefineXMLGeneratorPort) -> None:
        self._define_xml_generator_instance = generator


def create_default_container(verbose: int = 0) -> DependencyContainer:
    return DependencyContainer(verbose=verbose)
