Dependency injection with Rust - based on Spring. In somewhat stable state however aspects are not finished.

Aspects will be implemented (almost finished) as only augmenting those modules that are decorated.

In the future perhaps you can opt into the whole program being copied and aspects being created even without each module being decorated, this was the previous state, but codegen was really bad.  

Right now it has the dfactory codegen code generating but it hasn't imported knockoff_delegator_factories into knockoff_precompile_gen

knockoff_dfactory_gen generates knockoff_delegator_factories, which contains code to import into mutable macro
knockoff_delegator_factories needs to be imported into knockoff_precompile_gen, which will then be imported in the module_precompile_macro, which the user will import. 

So code is generated that is imported into module_precompile_macro, which is imported by the user.

