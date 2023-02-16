use knockoff_logging::{initialize_log, initialize_logger, use_logging};

use_logging!();
initialize_logger!(TextFileLogger, StandardLogData);

