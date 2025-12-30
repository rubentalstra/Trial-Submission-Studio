//! SDTM data model types per SDTMIG v3.4.

pub mod ct;
pub mod domain;
pub mod enums;

pub use ct::{Codelist, ResolvedCodelist, Term, TerminologyCatalog, TerminologyRegistry};
pub use domain::{DatasetClass, Domain, Variable, VariableType};
pub use enums::{CoreDesignation, VariableRole};
