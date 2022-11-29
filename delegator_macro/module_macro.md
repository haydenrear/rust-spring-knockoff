# Module Macro

Injector created from each module using attribute macro #[library] example. Users define particular types that implement
struct/fn, etc changing traits.

Inner modules must be in-lines, so that means that modules must be copied in-line if support for non-inline. Hygiene checker
must be implemented to check for hygiene, but supporting hygiene is out of scope. 

1. Base module gets all other modules copied in-line (pre-compilation)
2. Have all of context available for creating base injector.
3. Recursively create injectors for each module, and then aggregating injector

Hygiene checker can probably just be adding all identifiers to a map and making sure that there are no collisions.