# Overview Quick Note Get Some Notes Down

- Phase -> PhaseDependency: Phase contains other phases as deps****
- Phase -> ProgramDependency: Each phase contains program dependencies
- Phase -> Delegator: Each phase contains as input delegators, or produces delegators, which are used to create aggregate delegates
 
- ProgramFunnel -> Phase

- ModuleMacroCodegen -> Generate knockoff_providers_gen  
- ModulePrecompileGen -> Generate knockoff_precompile_gen 
- DFactoryDCodegen -> Code: User generate delegator based on build.rs, based on user generated code, d_factory_gen, number based on priority in factories.toml, activation based on existence in factories.toml 
- Delegator -> Delegate: The delegators delegate in each phase 
- FactoriesCodegen -> Code that generates delegator based on user provided code 

# Phases

- Providers: Able to use delegators to generate framework code from the user code, such as generating controller endpoints, but not modify the user code, (read-only attribute macro).
- PreCompile: Able to use delegators over the program to generate code to be imported into another part of the framework, which then gets exported by that part of the framework and used by the user. User override/customize framework.
- DFactory: Able to use delegators over the program to generate code to be imported by the user as a module macro which can modify the user's program. The user's code is imported by a module which generates a delegator to be then imported by a macro to modify user code.