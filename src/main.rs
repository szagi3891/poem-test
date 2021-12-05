use std::sync::Arc;

use poem::Response;
use poem::endpoint::make_sync;
use poem::{listener::TcpListener, Route, Server};
use poem_openapi::{payload::Json, Object, OneOf, OpenApi, OpenApiService};

use poem_openapi::ApiResponse;

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
        // CreateBlogResponse::InternalError

        CreateBlogResponse::Forbidden(Json(Forb {
            message: "dsadsa".into(),
            age: 4,
        }))
    }

    #[oai(path = "/put2", method = "post")]
    async fn index2(&self, obj: Json<MyObj>) -> Json<MyObj> {
        obj
    }
}

struct Api1 {}

#[OpenApi]
impl Api1 {
    pub fn new() -> Api1 {
        Api1 {}
    }

    #[oai(path = "/hello", method = "post")]
    async fn create_post(
        &self,
        obj: Json<MyObj>,
    ) -> CreateBlogResponse {

        CreateBlogResponse::Ok(Json(444))
    }
}

use serde_json::Value;

fn add_base(base: impl Into<String>, spec: &String) -> String {
    let base: String = base.into();

    let mut spec_value = serde_json::from_str::<Value>(spec.as_str()).unwrap();

    if let Value::Object(props) = &mut spec_value {
        if let Some(paths_inner) = props.get_mut("paths") {

            if let Value::Object(props2) = paths_inner {

                let mut new_map = serde_json::Map::new();
                let base = &base;

                for (key, item) in props2.iter() {
                    let new_name = format!("{base}{key}");
                    new_map.insert(new_name, item.clone());
                }

                *props2 = new_map;

            } else {
                panic!("dddd1..");
            }

        } else {
            panic!("dddd2..");
        }

    } else {
        panic!("ddddd..");
    }

    serde_json::to_string(&spec_value).unwrap()
}


#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }

    tracing_subscriber::fmt::init();

    let api1 = Api1::new();

    let api_service1 = OpenApiService::new(api1, "Oneof", "1.0").server("http://localhost:3000/api1");
    let spec1 = api_service1.spec_endpoint();


    let api2 = Api2::new();

    let api_service2 = OpenApiService::new(api2, "Oneof", "1.0").server("http://localhost:3000/api2");
    let spec2 = api_service2.spec_endpoint();



    let spec_endpoint = {
        let spec = api_service1.spec();
        let spec = add_base("/api1", &spec);

        make_sync(move |_| {
            Response::builder()
                .content_type("application/json")
                .body(spec.clone())
        })
    };



    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(Route::new()
            .nest("/spec1-correct.json", spec_endpoint)
            .nest("/spec1.json", spec1)
            .nest("/api1", api_service1)
            .nest("/spec2.json", spec2)
            .nest("/api2", api_service2)
        )
        .await
}
