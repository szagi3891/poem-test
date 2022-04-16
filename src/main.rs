use std::collections::HashMap;
use std::sync::Arc;

use poem::{listener::TcpListener, Route, Server};
use poem_openapi::payload::PlainText;
use poem_openapi::types::{ToJSON, ParseFromJSON};
use poem_openapi::{payload::Json, Object, OpenApi, OpenApiService, Enum};
use poem_openapi::{ApiResponse, Union};
use poem::{endpoint::StaticFilesEndpoint};

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

#[derive(Clone, Debug, PartialEq, Enum)]
enum GeoRuleType {
    DENY,
    ALLOW,
}

#[derive(Clone, Debug, PartialEq, Object)]
pub struct SportModel {
    pub id: Arc<String>,
    pub name: Arc<String>,
    pub display_order: u64,
    geo_rule_type: GeoRuleType,
}


#[derive(Object, Debug, PartialEq)]
struct B {
    v3: f32,
    list: HashMap<String, u64>,
    sport: Option<Arc<SportModel>>,
}

#[derive(Union, Debug, PartialEq)]
#[oai(discriminator_name = "type")]
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
    Forbidden(Json<Option<Forb>>),
  
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
    async fn create_post(&self, obj: Json<MyObj>) -> CreateBlogResponse {
        CreateBlogResponse::Forbidden(Json(Some(Forb {
            message: "dsadsa".into(),
            age: 4,
        })))
    }

    #[oai(path = "/put2", method = "post")]
    async fn index2(&self, obj: Json<MyObj>) -> Json<MyObj> {
        obj
    }

    #[oai(method = "get", path = "/fun1")]
    async fn fun1(&self) -> PlainText<String> {
        PlainText("response from fun1".into())
    }

    #[oai(method = "get", path = "/fun2")]
    async fn fun2(&self) -> PlainText<String> {
        PlainText("response from fun2".into())
    }

    #[oai(method= "get", path = "/fun3")]
    async fn fun3(&self) -> Json<ProxyResponse<String>> {
        // PlainText("response from fun2".into())
        todo!()
    }
    //ProxyResponse

    #[oai(method= "get", path = "/fun4")]
    async fn fun4(&self) -> Json<ProxyResponse<u32>> {
        // PlainText("response from fun2".into())
        todo!()
    }
    //ProxyResponse
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

    async fn call(&self, req: Request) -> Result<Self::Output, poem::Error> {
        let label = self.label.clone();
        let path = req.uri().path();

        println!("{label} => request: {path}");

        let resp = self.endpoint.call(req).await?;
        let resp = resp.into_response();

        if resp.status().is_success() {
            let status = resp.status();
            println!("{label} => response: {status}");
        } else {
            let status = resp.status();
            println!("{label} => error: {status}");
        }
        Ok(resp)
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

#[derive(Clone, Debug, PartialEq, Enum)]
#[oai(rename_all="PascalCase")]
enum SelectionDisplayType {
    Column,
    Row,
    TwoColumns,
    ThreeColumns,
    CorrectScore
}

#[derive(Clone, Debug, PartialEq, Object)]
struct RabViewDetailsModel {
    displayOrder: u64,
    selectionDisplayType: SelectionDisplayType,
    active: bool,
}

// #[test]
// fn decode_enum() {
//     let data = r#"{"displayOrder":2,"active":true,"selectionDisplayType":"CorrectScore"}"#;

//     let aa = serde_json::from_str::<RabViewDetailsModel>(&data);
//     println!("ddd {aa:#?}");

//     let g = RabViewDetailsModel {
//         displayOrder: 33,
//         selectionDisplayType: SelectionDisplayType::CorrectScore,
//         active: true
//     };

//     let hh = serde_json::to_string(&g);
//     println!("hh {hh:#?}");
// }


#[derive(Debug, Object)]
pub struct ProxyResponseOk<R: Send + ToJSON + ParseFromJSON> {
    pub response: R
}

#[derive(Debug, Object)]
pub struct ProxyResponseMessage {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Object)]
pub struct ProxyResponseUnknown {
    pub code: u32,
    pub body: String,
}

#[derive(Debug, Union)]
#[oai(discriminator_name="type")]
pub enum ProxyResponse<R: Send + ToJSON + ParseFromJSON> {
    Ok(ProxyResponseOk<R>),
    Error(ProxyResponseMessage),
    Unknown(ProxyResponseUnknown),
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
                StaticFilesEndpoint::new("./src").show_files_listing(),
            )

            .nest(
                "/static2",
                StaticFilesEndpoint::new("./src"),
            )

        )
        .await
}

