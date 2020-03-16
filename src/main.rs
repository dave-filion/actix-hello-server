use actix_web::*;
use listenfd::ListenFd;
use std::sync::Mutex;

// Struct of app data shared in scope
struct AppState {
    name: String,
    count: Mutex<i32>,  // requires mutex to share between threads
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Yo")
}

async fn index2() -> impl Responder {
    HttpResponse::Ok().body("Yo again!")
}

async fn inc_counter(data : web::Data<AppState>) -> String {
    let mut counter = data.count.lock().unwrap(); // block until gets lock
    *counter += 1;  // access counter inside mutex guard

    format!("Count num: {}", counter) // response with count
    // counter drops and mutex releases lock here
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
            .app_data(app_state.clone()) // register the created data
            .route("/", web::get().to(index))
            .route("/again", web::get().to(index2))
            .route("/inc", web::get().to(inc_counter))
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind("127.0.0.1:8088")?
    };
    println!("Server running!");

    server.run().await
}
