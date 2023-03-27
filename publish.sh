strings=(
  "codegen_utils"
  "wait_for"
  "knockoff_logging"
  "build_lib"
  "web_framework_shared"
  "crate_gen"
  "data_framework"
  "mongo_repo"
  "factories_codegen"
  "knockoff_security"
  "spring_knockoff_boot_macro"
  "module_macro_codegen"
  "module_macro_shared"
  "authentication_gen"
  "security_parse_provider"
  "handler_mapping"
  "web_framework"
)
for i in "${strings[@]}"; do
  cargo publish --registry=estuary --allow-dirty -p $i
done
cargo publish --registry=estuary --allow-dirty --no-verify -p module_macro_lib
cargo publish --registry=estuary --allow-dirty --no-verify -p module_macro
