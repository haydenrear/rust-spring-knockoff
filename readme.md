To build:

```shell
cd crategen_helper
cargo build
cd ../delegator_test
cargo build
// have to build twice...
cargo build
```

Because the module_macro_lib contains a reference to the generated crate to add code that the user provides, the module_macro_lib crate needs to be generated using a build script. However, the build script can't run if the module_macro_lib crate doesn't exist. So that means that either the build script needs to update the Cargo.toml or a shell script needs to run beforehand such as a gradle task, that would

1. generate the module_macro_lib based on the factories.toml that the user provides. 
2. generate a dummy knockoff_gen for the module_macro_lib to point to
3. build twice, first to update cargo.toml and generate code, second time to build with the generated code.