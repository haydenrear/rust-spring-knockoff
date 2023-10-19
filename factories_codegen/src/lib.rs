/**
At each stage of the injection, you have a plugin point.
1. ParseContainerModifier: decide which beans to add to the parse container.
2. ProfileTreeModifier: translating parse container to profile tree
3. ProfileTreeFinalizer: finalize ParseContainer and ProfileTree contained in ParseContainer for next
   step of codegen.
4. TokenProvider: adding to the context based on what was added to the parse container.
Example: HttpSecurity
- ParseContainerModifier: check to see if item contains annotation http_security and add to ParseContainer
  as Bean.
- ProfileTreeModifier: Add http_security BeanDefinition to ProfileTree.
- ProfileTreeFinalizer: Perform any post profile tree modification verification, or other, after all
  ProfileTreeModifiers have been run.
- TokenProvider: Add security filters and associated logic to be available for HandlerMapping.
 */
pub mod factories_parser;
pub mod token_provider;
pub mod parse_provider;
pub mod parse_container_modifier;
pub mod provider;
pub mod profile_tree_modifier;
pub mod profile_tree_finalizer;
pub mod item_modifier;
pub mod bean_type_providers;
pub mod logger;
pub mod test;
