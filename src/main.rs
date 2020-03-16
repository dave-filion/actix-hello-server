use actix_web::*;

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Yo Bro")
}

async fn index2() -> impl Responder {
    HttpResponse::Ok().body("Yo again!")
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/again", web::get().to(index2))
    })
        .bind("127.0.0.1:8088").unwrap()
        .run()
        .await
}
