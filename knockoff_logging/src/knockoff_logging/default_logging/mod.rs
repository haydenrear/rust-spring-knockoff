use crate::{initialize_log, initialize_logger, create_logger_expr, use_default_logging};
use codegen_utils::project_directory;

use_default_logging!();

initialize_logger!(TextFileLoggerImpl, StandardLogData, concat!(project_directory!(), "log_out/default_log.log"));