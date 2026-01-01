//! XPT header record parsing and building.
//!
//! This module handles the various header records in an XPT file:
//! - Library headers (file-level metadata)
//! - Member headers (dataset-level metadata)
//! - NAMESTR records (variable definitions)
//! - OBS header (marks start of observation data)

pub mod common;
pub mod datetime;
pub mod label;
pub mod library;
pub mod member;
pub mod namestr;

// Re-export common utilities and constants
pub use common::{
    DSCRPTR_HEADER_V5, DSCRPTR_HEADER_V8, LABELV8_HEADER, LABELV9_HEADER, LIBRARY_HEADER_V5,
    LIBRARY_HEADER_V8, MEMBER_HEADER_V5, MEMBER_HEADER_V8, NAMESTR_HEADER_V5, NAMESTR_HEADER_V8,
    NAMESTR_LEN, NAMESTR_LEN_VAX, OBS_HEADER_V5, OBS_HEADER_V8, RECORD_LEN, align_to_record,
    build_header_record, normalize_name, read_i16, read_i32, read_string, read_u16, records_needed,
    truncate_str, write_i16, write_i32, write_string,
};

// Re-export datetime utilities
pub use datetime::{format_xpt_datetime, parse_xpt_datetime};

// Re-export label section utilities
pub use label::{
    LabelSectionType, build_labelv8_data, build_labelv8_header, build_labelv9_data,
    build_labelv9_header, determine_label_section, is_label_header, is_labelv8_header,
    is_labelv9_header, parse_labelv8_data, parse_labelv9_data,
};

// Re-export library header utilities
pub use library::{LibraryInfo, build_library_header, build_real_header, build_second_header};

// Re-export member header utilities
pub use member::{
    build_dscrptr_header, build_member_data, build_member_header, build_member_second,
    build_namestr_header, build_obs_header, namestr_block_size, parse_dataset_label,
    parse_dataset_name, parse_dataset_type, parse_namestr_len, parse_variable_count,
};

// Re-export header validation
pub use library::{detect_version, validate_library_header};
pub use member::{
    validate_dscrptr_header, validate_member_header, validate_namestr_header, validate_obs_header,
};

// Re-export namestr utilities
pub use namestr::{build_namestr, parse_namestr, parse_namestr_records};
