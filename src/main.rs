extern crate actix;
extern crate actix_web;
// extern crate futures;
// extern crate dotenv;
extern crate juniper;

// use std::env;
use actix_web::{
    App, 
    // AsyncResponder,
    // Body,
    // client,
    Error, 
    // HttpMessage,
    HttpRequest, 
    // HttpResponse, 
    fs,
    middleware,
    server, 
};
// use actix_web::http::StatusCode;
use fs::NamedFile;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
// use futures::{Future, Stream};
// use dotenv::dotenv;

pub fn index(_req: &HttpRequest) -> Result<NamedFile, Error> {
    Ok(NamedFile::open("ui/dist/index.html")?)
}

fn main() {
    let host = "127.0.0.1:3000";
    // dotenv().ok();
    let sys = actix::System::new("upquark");
    let _addr = server::new(move || {
        App::new()
            .resource("/", |r| r.f(index))
            .handler("/img", fs::StaticFiles::new("ui/dist/img").expect("fail to handle static images"))
            .handler("/js", fs::StaticFiles::new("ui/dist/js").expect("fail to handle static js"))
            .handler("/css", fs::StaticFiles::new("ui/dist/css").expect("fail to handle static css"))
            .handler("/", fs::StaticFiles::new("ui/dist").expect("fail to handle static files"))
        })
    .bind(host).expect(&format!("could not bind to {}", host))
    .start();
        
    println!("Starting http server: {}", host);
    let _ = sys.run();
}