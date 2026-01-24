//! CSV reading utilities.

mod header;
mod reader;

pub use header::CsvHeaders;
pub use reader::{
    MAX_CSV_FILE_SIZE, check_file_size, check_file_size_with_limit, read_csv_schema,
    read_csv_table, validate_dataframe_shape, validate_encoding,
};
