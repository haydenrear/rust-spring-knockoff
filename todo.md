# TODO:

- [ ] split out the creation of the factories into a provider like the aspect
- [ ] split out authentication_type and field_aug into a provider like the aspect (and then remove module_macro_codegen entirely)
- [ ] create provider for ConfigurationProperties and properties abstraction to load properties hierarchically from files using Profiles and Priority.
- [ ] add application context initializer (see spring boot macro for info)
- [ ] add ability to pass arguments to prototype bean factory
- [ ] update the activation of git so that it doesn't recompile all files every time. 