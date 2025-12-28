//! Wide format data structures for LB, VS, and IE domains.

/// Wide format group for LB (Laboratory) domain columns.
#[derive(Debug, Default, Clone)]
pub struct LbWideGroup {
    pub base_key: String,
    pub test_col: Option<usize>,
    pub testcd_col: Option<usize>,
    pub orres_col: Option<usize>,
    pub orresu_col: Option<usize>,
    pub orresu_alt_col: Option<usize>,
    pub ornr_range_col: Option<usize>,
    pub ornr_lower_col: Option<usize>,
    pub ornr_upper_col: Option<usize>,
    pub range_col: Option<usize>,
    pub clsig_col: Option<usize>,
    pub date_col: Option<usize>,
    pub time_col: Option<usize>,
    pub extra_cols: Vec<usize>,
}

/// Wide format group for VS (Vital Signs) domain columns.
#[derive(Debug, Default, Clone)]
pub struct VsWideGroup {
    pub key: String,
    pub label: Option<String>,
    pub orres_col: Option<usize>,
    pub orresu_col: Option<usize>,
    pub pos_col: Option<usize>,
    pub extra_cols: Vec<usize>,
}

/// Shared columns for VS wide format (BP fallbacks).
#[derive(Debug, Default, Clone)]
pub struct VsWideShared {
    pub orresu_bp: Option<usize>,
    pub pos_bp: Option<usize>,
}

/// Wide format group for IE (Inclusion/Exclusion) domain columns.
#[derive(Debug, Default, Clone)]
pub struct IeWideGroup {
    pub category: String,
    pub test_col: Option<usize>,
    pub testcd_col: Option<usize>,
}

/// Suffix kinds for LB column pattern matching.
#[derive(Debug, Clone, Copy)]
pub enum LbSuffixKind {
    TestCd,
    Test,
    Orres,
    Orresu,
    OrresuAlt,
    OrnrRange,
    OrnrLower,
    OrnrUpper,
    Range,
    Clsig,
}
