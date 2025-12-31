//! XPT header record parsing and building.
//!
//! This module handles the various header records in an XPT file:
//! - Library headers (file-level metadata)
//! - Member headers (dataset-level metadata)
//! - NAMESTR records (variable definitions)
//! - OBS header (marks start of observation data)

pub mod datetime;
pub mod label;
pub mod library;
pub mod member;
pub mod namestr;

// Re-export commonly used items
pub use datetime::{format_xpt_datetime, parse_xpt_datetime};
pub use label::{
    LABELV8_HEADER_PREFIX, LABELV9_HEADER_PREFIX, LabelSectionType, build_labelv8_data,
    build_labelv8_header, build_labelv9_data, build_labelv9_header, determine_label_section,
    is_label_header, is_labelv8_header, is_labelv9_header, parse_labelv8_data, parse_labelv9_data,
};
pub use library::{
    LIBRARY_HEADER_PREFIX, LibraryInfo, RECORD_LEN, build_library_header, build_real_header,
    build_second_header, validate_library_header,
};
pub use member::{
    DSCRPTR_HEADER_PREFIX, MEMBER_HEADER_PREFIX, NAMESTR_HEADER_PREFIX, OBS_HEADER_PREFIX,
    align_to_record, build_dscrptr_header, build_member_data, build_member_header,
    build_member_second, build_namestr_header, build_obs_header, namestr_block_size,
    parse_dataset_label, parse_dataset_name, parse_dataset_type, parse_namestr_len,
    parse_variable_count, validate_dscrptr_header, validate_member_header, validate_namestr_header,
    validate_obs_header,
};
pub use namestr::{
    NAMESTR_LEN, NAMESTR_LEN_VAX, build_namestr, parse_namestr, parse_namestr_records,
};
