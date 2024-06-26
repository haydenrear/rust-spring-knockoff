# TODO: isn't currently in order...
strings=(
  "aspect_knockoff_provider"
  "authentication_gen"
  "authentication_codegen"
  "codegen_utils"
  "collection_util"
  "crate_gen"
  "data_framework"
  "factories_codegen"
  "handler_mapping"
  "knockoff_logging"
  "knockoff_security"
  "knockoff_helper"
  "knockoff_tokio_util"
  "module_macro_codegen"
  "module_macro_shared"
  "mongo_repo"
  "security_parse_provider"
  "spring_knockoff_boot"
  "spring_knockoff_boot_macro"
  "wait_for"
  "web_framework"
  "web_framework_shared"
  "string_utils"
  "set_enum_fields"
  "knockoff_env"
  "configuration_properties_macro"
  "module_precompile"
  "module_precompile_codegen"
  "knockoff_resource"
)
for i in "${strings[@]}"; do
  cargo publish --registry=estuary --allow-dirty --no-verify -p $i
done
cargo publish --registry=estuary --allow-dirty --no-verify -p module_macro_lib
cargo publish --registry=estuary --allow-dirty --no-verify -p module_macro
