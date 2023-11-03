To build:

```shell
chmod 777 build_cli.sh
./build_cli.sh
./build_dev.sh
cargo build
```

It will throw an error but the error is expected.

# Including it in another project

## Multiple workspaces

If you get a multiple workspaces issue, then that means you need to add the following to your Cargo.toml:

```toml
[workspace]
members = [
    # members that include
]
# The below is the important part
exclude = [
    "target/knockoff_providers_gen",
    "target/module_macro",
    "target/module_macro_lib"
]
```

## Including 

You need to add a build.rs:

```rust
use std::{env, fs};
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::ops::Deref;
use std::path::Path;
use std::ptr::write;
use syn::__private::{Span, ToTokens};
use syn::{braced, Fields, Ident, Item, ItemMod, ItemStruct, Token, token, Visibility, VisPublic};
use syn::__private::quote::__private::push_div_eq_spanned;
use syn::parse::{ParseBuffer, ParseStream};
use syn::token::Brace;
use build_lib::replace_modules;
use codegen_utils::env::{get_project_base_build_dir, get_build_project_dir};
use codegen_utils::project_directory;
use crate_gen::CrateWriter;

fn main() {
    replace_modules(
        Some(get_build_project_dir("SRC_DIRECTORY_HERE").as_str()),
        vec![get_project_base_build_dir().as_str()]
    );
}
```

You need to add a codegen_resources directory at root with knockoff_factories.toml:

```toml
[dependencies]
# Dependencies necessary to run the providers
quote = "1.0"
syn = {version = "1.0", features = ["full"]}
proc-macro2 = "1.0"
serde = "1.0.137"

[dependencies.web_framework]
name = "web_framework"
version = "0.1.5"
registry = "estuary"
[dependencies.web_framework_shared]
name = "web_framework_shared"
version = "0.1.5"
registry = "estuary"
[dependencies.module_macro_shared]
name = "module_macro_shared"
version = "0.1.5"
registry = "estuary"

# Providers for web endpoints
[token_provider.handler_mapping.provider_data]
provider_path = "handler_mapping::HandlerMappingBuilder"
provider_ident = "HandlerMappingTokenProvider"
[token_provider.handler_mapping.dependency_data]
version = "0.1.5"
registry = "estuary"

# Providers for authentication types
[parse_provider.security_parse_provider.provider_data]
provider_path = "security_parse_provider::SecurityParseProvider"
provider_ident = "SecurityParseProviderBuilder"
[parse_provider.security_parse_provider.dependency_data]
registry = "estuary"
version = "0.1.5"

# Providers for aspects
[item_provider.aspect_knockoff_provider.provider_data]
provider_path = "aspect_knockoff_provider::aspect_knockoff_provider::aspect_item_modifier::AspectParser"
provider_ident = "AspectParserBuilder"
[item_provider.aspect_knockoff_provider.dependency_data]
registry = "estuary"
version = "0.1.5"
[parse_provider.aspect_knockoff_provider.provider_data]
provider_path = "aspect_knockoff_provider::aspect_knockoff_provider::aspect_parse_provider::ParsedAspects"
provider_ident = "ParsedAspectsBuilder"
[parse_provider.aspect_knockoff_provider.dependency_data]
registry = "estuary"
version = "0.1.5"
[token_provider.aspect_knockoff_provider.provider_data]
provider_path = "aspect_knockoff_provider::aspect_knockoff_provider::aspect_ts_generator::AspectGenerator"
provider_ident = "AspectGeneratorBuilder"
[token_provider.aspect_knockoff_provider.dependency_data]
registry = "estuary"
version = "0.1.5"
[bean_ty_provider.aspect_knockoff_provider.provider_data]
provider_path = "aspect_knockoff_provider::aspect_knockoff_provider::AspectInfo"
provider_ident = "AspectInfoBuilder"
[bean_ty_provider.aspect_knockoff_provider.dependency_data]
registry = "estuary"
version = "0.1.5"
```

You need to add a .cargo/config.toml like the following:

```toml
[env]
LOGGING_DIR = "Logging file"
PROJECT_BASE_DIRECTORY = "DIRECTORY OF WORKSPACE CARGO.TOML"
MODULE_MACRO_REGISTRY_INDEX_URI = "http://localhost:1234/git/index"
[registries]
# registry with the modules
estuary = { index = "http://localhost:1234/git/index" }
```

You need to add the following dependencies # TODO:

```toml
[dependencies]
paste = "1.0.12"
serde = "1.0.137"

[dependencies.module_macro_shared]
version = "0.1.5"
registry = "estuary"
[dependencies.web_framework]
version = "0.1.5"
registry = "estuary"
[dependencies.web_framework_shared]
version = "0.1.5"
registry = "estuary"
[dependencies.module_macro]
path = "../target/module_macro"
version = "0.1.5"
[dependencies.module_macro_lib]
path = "../target/module_macro_lib"
version = "0.1.5"


[build-dependencies]
syn = {version = "1.0", features = ["full"]}
paste = "1.0.12"
lazy_static = "1.4.0"
[build-dependencies.crate_gen]
version = "0.1.5"
registry = "estuary"
[build-dependencies.web_framework]
version = "0.1.5"
registry = "estuary"
[build-dependencies.spring_knockoff_boot_macro]
version = "0.1.5"
registry = "estuary"
[build-dependencies.build_lib]
version = "0.1.5"
registry = "estuary"
[build-dependencies.module_macro_codegen]
version = "0.1.5"
registry = "estuary"
[build-dependencies.module_macro_shared]
version = "0.1.5"
registry = "estuary"
[build-dependencies.codegen_utils]
version = "0.1.5"
registry = "estuary"
[build-dependencies.knockoff_logging]
version = "0.1.5"
registry = "estuary"
```

Also you need to add the following so that it can add the beans:

```rust

use std::any::{Any, TypeId};
use std::collections::{HashMap, LinkedList};
use std::ops::Deref;
use std::ptr::slice_from_raw_parts;
use std::sync::Arc;
use std::marker::PhantomData;


include!(concat!(env!("OUT_DIR"), "/spring-knockoff.rs"));

use module_macro::module_attr;

/// This works by copying all of the submodules and in the process creating the bean factory 
#[module_attr]
#[cfg(springknockoff)]
pub mod root_module {
    /// beans go here or in submodules declared in-line or not in-line (macro makes them in-line)
}
```

For more example check delegator_test crate in this repo.

And then you need to install the CLI

If you are running it in another project (not changing files in this directory) then you run the cli like the following:

```shell
knockoff_cli --registry-uri=http://localhost:1234/git/index --mode=dev
```

Otherwise use the above

# Estuary

To include in another project you need to:

1. Start estuary in Docker:

```shell
# if you accidentally need to redeploy it, then don't forget to clear your cargo cache, and 
rm -rf ~/estuary_data/
# if you get an error afterwards about a hash, then that's actually your Cargo.lock.
rm -rf ~/.cargo/registry/cache/localhost-*
docker build -t estuary-quickstart .
docker run --rm -it -p 1234:7878 -v ~/estuary_data:/var/lib/estuary estuary-quickstart --base-url=http://localhost:1234
```

Make sure you have in your .cargo/config.toml in root:

```toml
[registries]
estuary = { index = "http://localhost:1234/git/index" }
```

## Example Yank

Say you deploy and then need to redeploy a particular module. So you need to yank from estuary.

```shell
cargo yank [package_name] --version [version] --registry estuary
# OR
cargo yank package_name@version --registry estuary
```

This assumes that you have

```toml
[registries]
estuary = { index = "http://localhost:1234/git/index" }
```

2. Publish the packages

```shell
chmod +x publish_dirty.sh
./publish_dirty.sh
```

Note this doesn't compile so if you want to compile then:

```shell
chmod +x publish.sh
./publish.sh
```

3. Use in the above Cargo.toml

