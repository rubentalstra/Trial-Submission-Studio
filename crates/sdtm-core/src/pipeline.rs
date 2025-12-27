//! Domain processing pipeline with ordered step execution.
//!
//! This module provides a step-based pipeline executor for domain processing.
//! Each step implements the `ProcessingStep` trait and is executed in order.
//!
//! # Standard Pipeline Order
//!
//! 1. **BaseRulesStep** - Apply base rules (USUBJID prefixing)
//! 2. **DomainProcessorStep** - Run domain-specific processor
//! 3. **CtNormalizationStep** - Normalize controlled terminology values
//! 4. **SequenceAssignmentStep** - Assign --SEQ values
//!
//! # Example
//!
//! ```ignore
//! use sdtm_core::pipeline::{DomainPipeline, build_default_pipeline};
//!
//! let pipeline = build_default_pipeline();
//! pipeline.execute(&domain, &mut df, &ctx)?;
//! ```

use std::collections::BTreeMap;

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::domain_processors::{self, DomainProcessorRegistry};
use crate::processing_context::ProcessingContext;

/// A single processing step in the domain pipeline.
///
/// Each step performs a specific transformation or validation on the DataFrame.
pub trait ProcessingStep: Send + Sync {
    /// Execute this step on the domain DataFrame.
    ///
    /// # Arguments
    /// * `domain` - Domain metadata from standards
    /// * `df` - DataFrame to process (modified in place)
    /// * `ctx` - Processing context with study metadata
    /// * `state` - Mutable pipeline state for cross-step data sharing
    fn execute(
        &self,
        domain: &Domain,
        df: &mut DataFrame,
        ctx: &ProcessingContext,
        state: &mut PipelineState,
    ) -> Result<()>;

    /// Human-readable name for this step (for logging/debugging).
    fn step_name(&self) -> &str;

    /// Whether this step should be skipped based on context.
    ///
    /// Default implementation always runs the step.
    fn should_skip(&self, _domain: &Domain, _ctx: &ProcessingContext) -> bool {
        false
    }
}

/// Mutable state shared across pipeline steps.
///
/// This allows steps to share data without tight coupling.
#[derive(Default)]
pub struct PipelineState {
    /// Sequence tracker for --SEQ assignment across files.
    pub seq_tracker: Option<BTreeMap<String, i64>>,
    /// Custom processor registry (if not using default).
    pub processor_registry: Option<DomainProcessorRegistry>,
    /// Step execution log for debugging.
    pub executed_steps: Vec<String>,
}

impl PipelineState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_seq_tracker(mut self, tracker: BTreeMap<String, i64>) -> Self {
        self.seq_tracker = Some(tracker);
        self
    }

    pub fn with_registry(mut self, registry: DomainProcessorRegistry) -> Self {
        self.processor_registry = Some(registry);
        self
    }
}

/// An ordered pipeline of processing steps.
///
/// Steps are executed in order, allowing for flexible composition
/// of domain processing logic.
pub struct DomainPipeline {
    steps: Vec<Box<dyn ProcessingStep>>,
}

impl Default for DomainPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainPipeline {
    /// Create an empty pipeline.
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    /// Add a step to the end of the pipeline.
    pub fn add_step(mut self, step: Box<dyn ProcessingStep>) -> Self {
        self.steps.push(step);
        self
    }

    /// Insert a step at a specific position.
    pub fn insert_step(mut self, index: usize, step: Box<dyn ProcessingStep>) -> Self {
        self.steps.insert(index, step);
        self
    }

    /// Remove a step by name.
    pub fn remove_step(mut self, step_name: &str) -> Self {
        self.steps.retain(|s| s.step_name() != step_name);
        self
    }

    /// Execute all steps in order.
    pub fn execute(
        &self,
        domain: &Domain,
        df: &mut DataFrame,
        ctx: &ProcessingContext,
    ) -> Result<()> {
        let mut state = PipelineState::new();
        self.execute_with_state(domain, df, ctx, &mut state)
    }

    /// Execute all steps with provided state.
    pub fn execute_with_state(
        &self,
        domain: &Domain,
        df: &mut DataFrame,
        ctx: &ProcessingContext,
        state: &mut PipelineState,
    ) -> Result<()> {
        for step in &self.steps {
            if step.should_skip(domain, ctx) {
                continue;
            }
            step.execute(domain, df, ctx, state)?;
            state.executed_steps.push(step.step_name().to_string());
        }
        Ok(())
    }

    /// List step names in execution order.
    pub fn step_names(&self) -> Vec<&str> {
        self.steps.iter().map(|s| s.step_name()).collect()
    }
}

// ============================================================================
// Standard Processing Steps
// ============================================================================

/// Step 1: Apply base rules (USUBJID prefixing with study ID).
pub struct BaseRulesStep;

impl ProcessingStep for BaseRulesStep {
    fn execute(
        &self,
        domain: &Domain,
        df: &mut DataFrame,
        ctx: &ProcessingContext,
        _state: &mut PipelineState,
    ) -> Result<()> {
        crate::processor::apply_base_rules(domain, df, ctx)
    }

    fn step_name(&self) -> &str {
        "base_rules"
    }

    fn should_skip(&self, _domain: &Domain, ctx: &ProcessingContext) -> bool {
        !ctx.options.prefix_usubjid
    }
}

/// Step 2: Run domain-specific processor.
pub struct DomainProcessorStep;

impl ProcessingStep for DomainProcessorStep {
    fn execute(
        &self,
        domain: &Domain,
        df: &mut DataFrame,
        ctx: &ProcessingContext,
        state: &mut PipelineState,
    ) -> Result<()> {
        if let Some(registry) = &state.processor_registry {
            crate::domain_processors::process_domain_with_registry(domain, df, ctx, registry)
        } else {
            domain_processors::process_domain(domain, df, ctx)
        }
    }

    fn step_name(&self) -> &str {
        "domain_processor"
    }
}

/// Step 3: Normalize controlled terminology values.
pub struct CtNormalizationStep;

impl ProcessingStep for CtNormalizationStep {
    fn execute(
        &self,
        domain: &Domain,
        df: &mut DataFrame,
        ctx: &ProcessingContext,
        _state: &mut PipelineState,
    ) -> Result<()> {
        crate::processor::normalize_ct_columns(domain, df, ctx)
    }

    fn step_name(&self) -> &str {
        "ct_normalization"
    }

    fn should_skip(&self, _domain: &Domain, ctx: &ProcessingContext) -> bool {
        ctx.ct_registry.is_none()
    }
}

/// Step 4: Assign --SEQ values.
pub struct SequenceAssignmentStep;

impl ProcessingStep for SequenceAssignmentStep {
    fn execute(
        &self,
        domain: &Domain,
        df: &mut DataFrame,
        ctx: &ProcessingContext,
        state: &mut PipelineState,
    ) -> Result<()> {
        crate::processor::assign_sequence(domain, df, ctx, state.seq_tracker.as_mut())
    }

    fn step_name(&self) -> &str {
        "sequence_assignment"
    }

    fn should_skip(&self, _domain: &Domain, ctx: &ProcessingContext) -> bool {
        !ctx.options.assign_sequence
    }
}

/// Build the default domain processing pipeline.
///
/// This returns a pipeline with the standard SDTM processing steps:
/// 1. Base rules (USUBJID prefixing)
/// 2. Domain-specific processor
/// 3. CT normalization
/// 4. Sequence assignment
pub fn build_default_pipeline() -> DomainPipeline {
    DomainPipeline::new()
        .add_step(Box::new(BaseRulesStep))
        .add_step(Box::new(DomainProcessorStep))
        .add_step(Box::new(CtNormalizationStep))
        .add_step(Box::new(SequenceAssignmentStep))
}
