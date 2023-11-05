use knockoff_logging::import_logger_root;

/**
There are phases, and in the phases there are crates created that have functions that generate code.
These are the Delegating providers. The Delegating providers are then used when parsing the user's
program. For example, every syn::Item that gets parsed also gets passed to a hook for the user to
do whatever. The user can, for instance, save some data in the container for use in another stage.

The first phase is Phase::PreCompile. The precompile phase exists to be able to use user code in
other framework libraries. For example, if there is a web library in the framework, that web
framework library could need to depend on some user code. So then the user annotates some of their
code with a #[authentication_type], which is a processor. Then, the authentication_codegen crate
is programmed to parse the mods that are decorated with this processor. The user provides a toml
configuration file with the name of the file that contains the mod that has the authentication type.
Then, during build, the authentication_gen crate, which the web framework depends on, parses that
file and generates the exported values, by calling the codegen Delegators. The web framework then
depends on authentication_gen, which, in it's lib.rs, imports the code that it generated during
build.

The second phase is Provider. The Providers are not imported into other framework libraries, but
are instead generating code for the user's crate. The same codegen code is used, and the same
interfaces are implemented. However, the crucial differences are that Provider generates code for
the user crate, while Precompile generates code for other framework libraries. Moreover, the
Provider codegen parses all of the user code, using module_attr, and calls all of the providers.
The changes to the parsed items reflect in the user code. Examples of this are adding a field to a
struct, or implementing an aspect.

Within each of these phases exist stages. Each stage has it's own dependencies, and it's own crate,
and then these stage crates are imported into knockoff_providers_gen and knockoff_precompile_gen
and provided to the module_macro_lib, which generates code for the user's crate using a macro, and
module_precompile, which is instead called in build.rs of the crates that are imported by the
framework crates. In the future, the stages will be able to use the code from the previous stages,
as additional providers will be able to be added.

This simulates service provider and is the foundation for all additional frameworks built on top,
as it provides points of customization for the bean container, and points of customization for
arbitrary codegen.
 */
pub mod factories_parser;
pub mod token_provider;
pub mod parse_provider;
pub mod parse_container_modifier;
pub mod provider;
pub mod profile_tree_modifier;
pub mod profile_tree_finalizer;
pub mod item_modifier;
pub mod test;
pub mod framework_token_provider;


use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/factories_codegen.log"));
