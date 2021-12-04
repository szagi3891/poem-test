use std::sync::Arc;

use my_obj2::MyObj2;
use poem::{listener::TcpListener, Route, Server};
use poem_openapi::{payload::Json, Object, OneOf, OpenApi, OpenApiService};

use poem_openapi::ApiResponse;

mod my_obj2;

#[derive(Object, Debug, PartialEq)]
struct A {
    v1: i32,
    v2: String,
}

#[derive(Object, Debug, PartialEq)]
struct B {
    v3: f32,
}

#[derive(OneOf, Debug, PartialEq)]
#[oai(property_name = "type")]
enum MyObj {
    A(A),
    B(B),
}


#[derive(Object, Debug, PartialEq)]
struct Forb {
    message: String,
    age: i32,
}

#[derive(ApiResponse)]
enum CreateBlogResponse {
    /// Created successfully
    #[oai(status = 200)]
    Ok(Json<u64>),
    
    /// Permission denied
    #[oai(status = 403)]
    Forbidden(Json<Forb>),
  
    /// Internal error
    #[oai(status = 500)]
    InternalError,
}

use tokio::sync::Mutex;

#[derive(Clone)]
struct State {
    list: Arc<Mutex<Vec<String>>>,
}

impl State {
    pub fn new() -> State {
        State {
            list: Arc::new(Mutex::new(Vec::new()))
        }
    }
}

struct Api2 {
    state: State,
}

impl Api2 {
    pub fn new() -> Api2 {
        Api2 {
            state: State::new()
        }
    }
}

#[OpenApi]
impl Api2 {
    #[oai(path = "/hello", method = "get")]
    async fn create_post(
        &self,
        obj: Json<MyObj>,
    ) -> CreateBlogResponse {
        // match req {
        //     CreatePostRequest::Json(Json(blog)) => {
        //         todo!();
        //     }
        //     CreatePostRequest::Text(content) => {
        //         todo!();
        //     }
        // }

        // CreateBlogResponse::InternalError

        CreateBlogResponse::Forbidden(Json(Forb {
            message: "dsadsa".into(),
            age: 4,
        }))
    }

    #[oai(path = "/put2", method = "post")]
    async fn index2(&self, obj: Json<MyObj2>) -> Json<MyObj2> {
        obj
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }

    tracing_subscriber::fmt::init();

    let api2 = Api2::new();

    let api_service2 = OpenApiService::new(api2, "Oneof", "1.0").server("http://localhost:3000/api2");
    let ui2 = api_service2.swagger_ui();
    // Enable the OpenAPI specification
    let spec2 = api_service2.spec_endpoint();

    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(Route::new()
            .nest("/spec2.json", spec2)
            .nest("/api2", api_service2)
            .nest("/ui2", ui2)
        )
        .await
}
