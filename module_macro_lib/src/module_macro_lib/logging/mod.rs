use knockoff_logging::{initialize_log, initialize_logger, use_logging, create_logger_expr};
use codegen_utils::project_directory;

use_logging!();
initialize_logger!(
    TextFileLoggerImpl,
    StandardLogData,
    concat!(project_directory!(), "log_out/web_framework.log")
);

