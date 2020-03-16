use actix_web::*;
use actix_files::NamedFile;

use listenfd::ListenFd;
use std::sync::Mutex;
use futures::future::{ready, Ready};
use serde::{Serialize, Deserialize};
use std::io;
use failure::Fail;

// Struct of app data shared in scope
struct AppState {
    name: String,
    count: Mutex<i32>,  // requires mutex to share between threads
}

// Struct of an object we respond with
#[derive(Serialize)]
struct AppResponseObject {
    name: &'static str,
    success: bool,
}

// Example struct to be serialized
#[derive(Serialize)]
struct AppJsonResponse {
    success: bool,
    user_id: u32,
    name: &'static str,
}

// struct representing some event sent to app
#[derive(Deserialize, Serialize)]
struct Event {
    id: Option<i32>,
    timestamp: f64,
    kind: String,
    tags: Vec<String>,
}

// fake fn to represent adding data t db
fn store_in_db(timestamp: f64, kind : &String, tags: &Vec<String>) -> Event {
    println!("Adding to db: timestamp={:?} kind={:?} tags={:?}", timestamp, kind, tags);
    // generate id
    Event {
        id: Some(123),
        timestamp: timestamp,
        kind: kind.clone(),
        tags: tags.clone(),
    }
}

// handler to capture event, sent by json message
async fn capture_event(evt: web::Json<Event>) -> impl Responder {
    let new_event = store_in_db(evt.timestamp, &evt.kind, &evt.tags);
    format!("got event {}", new_event.id.unwrap())
}

#[derive(Serialize, Deserialize)]
struct MyInfo {
    user_id: u32,
    name: String,
}

async fn extractor_test(
    path_info : web::Path<MyInfo>, // data pulled from route/path
) -> Result<String> {
    Ok(format!("{} {}", path_info.user_id, path_info.name))
}

#[derive(Serialize, Deserialize)]
struct Info2 {
    name: String,
}

// get at query params
async fn query_test(info: web::Query<Info2>) -> String {
    format!("Welcome: {}", info.name)
}

// JSON extractor test
async fn json_test(info : web::Json<Info2>) -> String {
    println!("Into json handler");
    format!("Welcome: {}", info.name)
}

#[derive(Deserialize)]
struct FormData {
    username: String,
    number: u32,
}

// Form extractor test
async fn form_test(info: web::Form<FormData>) -> Result<String> {
    Ok(format!("Welcome {} -> {}!", info.username, info.number))
}

// Need to implement Responder for our objct so a handler can return it
impl Responder for AppResponseObject {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();

        // Create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

// Loads static file for index
async fn index(_req : HttpRequest) -> Result<NamedFile> {
    Ok(NamedFile::open("index.html")?)
}


async fn index2() -> impl Responder {
    HttpResponse::Ok().body("Yo again!")
}

async fn api_response() -> impl Responder {
    AppResponseObject{
        name: "JIMMY",
        success: true,
    }
}

async fn inc_counter(data : web::Data<AppState>) -> String {
    let mut counter = data.count.lock().unwrap(); // block until gets lock
    *counter += 1;  // access counter inside mutex guard

    format!("Count num: {}", counter) // response with count
    // counter drops and mutex releases lock here
}

// custom error example
#[derive(Fail, Debug)]
#[fail(display = "some error yo")]
struct MyError {
    name: &'static str,
}

// need to impl response error for custom struct, use default her
impl error::ResponseError for MyError {}

async fn error_test() -> Result<&'static str, MyError> {
    Err(MyError{ name: "error_test"})
}


// this function could be located in different module
// its scoped under /api
pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/test")
            .route(web::get().to(api_response))
            .route(web::head().to(|| HttpResponse::MethodNotAllowed())),
    );
}

// this function could be located in different module
fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/app")
            .route(web::get().to(|| HttpResponse::Ok().body("app")))
            .route(web::head().to(|| HttpResponse::MethodNotAllowed())),
    );
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    println!("Initializing server...");

    // start listener (for hot reloading in dev)
    let mut listenfd = ListenFd::from_env();

    // initialize app state data
    let app_state = web::Data::new(AppState {
        name: String::from("Dave"),
        count: Mutex::new(0),
    });

    let mut server =
        HttpServer::new(move || {
        // move app counter into closure

        App::new()
            .configure(config)
            .app_data(app_state.clone()) // register the created data
            // Configure info json extractor with max size
            .app_data(web::Json::<Info2>::configure(|cfg| {
                cfg.limit(4096).error_handler(|err, _req| {
                    // create custom error response
                    error::InternalError::from_response(
                        err,
                        HttpResponse::Conflict().finish(),

                    ).into()
                })
            }))
            .route("/", web::get().to(index))
            .route("/api", web::get().to(api_response))
            .route("/again", web::get().to(index2))
            .route("/inc", web::get().to(inc_counter))
            .route("/event", web::post().to(capture_event))
            .route("/extractor/{user_id}/{name}", web::get().to(extractor_test))
            .route("/query", web::get().to(query_test))
            .route("/json", web::post().to(json_test))
            .route("/form", web::post().to(form_test))
            .route("/error", web::get().to(error_test))
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind("127.0.0.1:8088")?
    };
    println!("Server running!");

    server.run().await
}
