use codegen_utils::project_directory;
use knockoff_logging::{initialize_log, initialize_logger, use_logging, create_logger_expr};

use_logging!();
initialize_logger!(TextFileLoggerImpl, StandardLogData, concat!(project_directory!(), "log_out/module_macro_codegen.log"));
