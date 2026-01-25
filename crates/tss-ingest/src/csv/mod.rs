//! CSV reading utilities.

mod header;
mod reader;

pub use header::CsvHeaders;
pub use reader::{
    EncodingResult, MAX_CSV_FILE_SIZE, check_file_size, check_file_size_with_limit,
    check_path_length, detect_and_transcode, read_csv_schema, read_csv_table,
    validate_dataframe_shape,
};
