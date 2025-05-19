# TODO: BROKEN WITH NEW CHANGES TO CARGO NOW
#knockoff_cli --mode=knockoff_dev
cargo +nightly build

#cargo clean || true

cd dfactory_dcodegen_codegen
cargo build
cd ../module_precompile_codegen
cargo build
cd ..
cargo test --package delegator_test --lib -- test_with_filter_chain --exact