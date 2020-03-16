use actix_web::*;
use listenfd::ListenFd;

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Yo")
}

async fn index2() -> impl Responder {
    HttpResponse::Ok().body("Yo again!")
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    println!("Initializing server...");

    // start listener (for hot reloading in dev)
    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/again", web::get().to(index2))
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    } else {
        server.bind("127.0.0.1:8088")?
    };
    println!("Server running!");

    server.run().await
}
