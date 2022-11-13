```puml

component Http
component WebFilter
component WebFramework
component HttpMacro

WebFilter -> Http: Http uses\nfilter
Http --> HttpMacro: HttpMacro\ndynamically creates\nendpoints
Http -> WebFramework
HttpMacro -> WebFramework

```

```puml

class Filter {
    void filter(HttpRequest, HttpResponse, FilterChain)
}

class EndpointMetadata {
    [str] path_variables
    [str] query_params
}

class GetController<T extends Serializable, U extends EndpointMetadata> {
    T get((HttpRequest, EndpointMetadata) -> T)
}

GetController --o Filter: filter calls \ncontroller endpoints

class PostController

class ControllerAttributeMacro {}
class Controller {}

ControllerAttributeMacro --o Controller: Annotated user \nfunction (HttpRequest) -> T\nis passed in as controller logic



```

When using procedural macro, how will the context include? We define a function that is similar to "play" - and that function is generated at compile time to contain the creation of the filters that include the calls to the user-defined structs. 