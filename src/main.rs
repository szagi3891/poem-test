use std::collections::HashMap;
use std::sync::Arc;

use poem::{listener::TcpListener, Route, Server};
use poem_openapi::{payload::Json, Object, OneOf, OpenApi, OpenApiService};
use poem_openapi::ApiResponse;
use poem::{endpoint::Files};

use poem::{
    async_trait, Endpoint, EndpointExt, IntoResponse,
    Middleware, Request, Response,
};

use poem::{http::Method, middleware::Cors};

#[derive(Object, Debug, PartialEq)]
struct A {
    v1: i32,
    v2: Arc<String>,
}


#[derive(Clone, Debug, PartialEq, Object)]
pub struct SportModel {
    pub id: Arc<String>,
    pub name: Arc<String>,
    pub displayOrder: u64
}

#[derive(Object, Debug, PartialEq)]
struct B {
    v3: f32,
    list: HashMap<String, u64>,
    sport: Option<Arc<SportModel>>,
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


struct Api1 {}

#[OpenApi(prefix_path = "/api1")]
impl Api1 {
    pub fn new() -> Api1 {
        Api1 {}
    }

    #[oai(path = "/hello", method = "post")]
    async fn create_post(
        &self,
        obj: Json<MyObj>,
    ) -> CreateBlogResponse {
        // log::info!("aaaa");
        println!("aaaaa {obj:#?}");
        CreateBlogResponse::Ok(Json(555))
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

#[OpenApi(prefix_path = "/api2")]
impl Api2 {
    #[oai(path = "/hello", method = "get")]
    async fn create_post(
        &self,
        obj: Json<MyObj>,
    ) -> CreateBlogResponse {
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



struct Log {
    label: Arc<String>,
}

impl Log {
    pub fn new(label: impl Into<String>) -> Log {
        Log {
            label: Arc::new(label.into())
        }
    }
}

impl<E: Endpoint> Middleware<E> for Log {
    type Output = LogImpl<E>;

    fn transform(&self, ep: E) -> Self::Output {
        LogImpl::new(ep, self.label.clone())
    }
}

struct LogImpl<E>{
    endpoint: E,
    label: Arc<String>
}

impl<E> LogImpl<E> {
    pub fn new(endpoint: E, label: Arc<String>) -> LogImpl<E> {
        LogImpl {
            endpoint,
            label
        }
    }
}

#[async_trait]
impl<E: Endpoint> Endpoint for LogImpl<E> {
    type Output = Response;

    async fn call(&self, req: Request) -> Self::Output {
        let label = self.label.clone();
        let path = req.uri().path();

        println!("{label} => request: {path}");

        let resp = self.endpoint.call(req).await.into_response();

        if resp.status().is_success() {
            let status = resp.status();
            println!("{label} => response: {status}");
        } else {
            let status = resp.status();
            println!("{label} => error: {status}");
        }
        resp
    }
}

fn cors() -> Cors {
    let cors = Cors::new()
        .allow_method(Method::GET)
        .allow_method(Method::POST)
        // .allow_origin("*")
        .allow_credentials(false)
    ;
    cors
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }

    tracing_subscriber::fmt::init();

    let api1 = Api1::new();
    let api_service1 = OpenApiService::new(api1, "Oneof", "1.0");
    // let spec1 = api_service1.spec();
    let spec1 = api_service1.spec_endpoint();

    let api2 = Api2::new();
    let api_service2 = OpenApiService::new(api2, "Oneof", "1.0");
    // let spec2 = api_service2.spec();
    let spec2 = api_service2.spec_endpoint();


    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(Route::new()
            .nest("/spec1.json", spec1) //spec_endpoint("/api1", &spec1))
            .nest_no_strip("/api1", 
                api_service1
                    .with(Log::new("łotr jeden"))
                    .with(cors())
            )
            .nest("/spec2.json", spec2) //spec_endpoint("/api2", &spec2))
            .nest_no_strip("/api2",
                api_service2
                .with(Log::new("rozkaz rozkaz"))
                .with(cors())
            )

            .nest(
                "/static1",
                Files::new("./src").show_files_listing(),
            )

            .nest(
                "/static2",
                Files::new("./src"),
            )

        )
        .await
}

