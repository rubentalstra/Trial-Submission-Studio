use std::env;

use sdtm_cli::logging::init_logging;

fn main() {
    let mut verbosity: u8 = 0;
    for arg in env::args().skip(1) {
        if arg.starts_with('-') && arg.chars().skip(1).all(|c| c == 'v') {
            verbosity = verbosity.saturating_add((arg.len() - 1) as u8);
        }
    }
    init_logging(verbosity);
    tracing::info!("sdtm-cli initialized");
}
