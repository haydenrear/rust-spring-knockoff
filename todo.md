# TODO:
- [ ] add generics provider to add generics support for the creation of the factories and add the ability to create factories with generic parameters. For example, currently the framework maps dyn type to concrete type that implements. the desired functionality would map generic parameters that are not implemented to dyn types, and then map those dyn types to concrete types when the dyn type is reference when getting from the bean factory. 
- [ ] when a user provides #[autowired] on a bean that is not Arc or Arc<Mutex then automatically use the ProtoypeFactory to inject it.
- [ ] add the ability to create an ordering to be implemented for each of the providers, and have this ordering be a consumer of all previous orderings and output a new ordering. For example, create a provider that accepts the ID's of a provider type, and has the ability to change the orderings of the providers.
- [ ] split out the creation of the factories into a provider like the aspect
- [ ] split out authentication_type and field_aug into a provider like the aspect (and then remove module_macro_codegen entirely)
- [ ] create provider for ConfigurationProperties and properties abstraction to load properties hierarchically from files using Profiles and Priority.
- [ ] add application context initializer (see spring boot macro for info)
- [ ] add ability to pass arguments to prototype bean factory
- [ ] update the activation of git so that it doesn't recompile all files every time. 