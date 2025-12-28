//! Tests for study pipeline context and domain pipeline.

use polars::prelude::DataFrame;
use sdtm_core::pipeline::{PipelineState, ProcessingStep, build_default_pipeline};
use sdtm_core::processing_context::ProcessingContext;
use sdtm_core::study_pipeline_context::StudyPipelineContext;
use sdtm_model::Domain;
use std::collections::BTreeMap;

// ============================================================================
// StudyPipelineContext Tests
// ============================================================================

#[test]
fn creates_pipeline_context() {
    let ctx = StudyPipelineContext::new("TEST001");
    assert_eq!(ctx.study_id, "TEST001");
    assert!(ctx.standards.is_empty());
}

#[test]
fn creates_processing_context() {
    let mut ref_starts = BTreeMap::new();
    ref_starts.insert("SUBJ001".to_string(), "2024-01-01".to_string());

    let pipeline = StudyPipelineContext::new("TEST001").with_reference_starts(ref_starts);

    let ctx = pipeline.processing_context();
    assert_eq!(ctx.study_id, "TEST001");
    assert!(ctx.reference_starts.is_some());
}

// ============================================================================
// DomainPipeline Tests
// ============================================================================

#[test]
fn default_pipeline_has_standard_steps() {
    let pipeline = build_default_pipeline();
    let names = pipeline.step_names();

    assert_eq!(names.len(), 4);
    assert_eq!(names[0], "base_rules");
    assert_eq!(names[1], "domain_processor");
    assert_eq!(names[2], "ct_normalization");
    assert_eq!(names[3], "sequence_assignment");
}

#[test]
fn pipeline_can_add_and_remove_steps() {
    struct CustomStep;
    impl ProcessingStep for CustomStep {
        fn execute(
            &self,
            _domain: &Domain,
            _df: &mut DataFrame,
            _ctx: &ProcessingContext,
            _state: &mut PipelineState,
        ) -> anyhow::Result<()> {
            Ok(())
        }
        fn step_name(&self) -> &str {
            "custom"
        }
    }

    let pipeline = build_default_pipeline()
        .add_step(Box::new(CustomStep))
        .remove_step("ct_normalization");

    let names = pipeline.step_names();
    assert_eq!(names.len(), 4);
    assert!(names.contains(&"custom"));
    assert!(!names.contains(&"ct_normalization"));
}
