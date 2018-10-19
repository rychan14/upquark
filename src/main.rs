extern crate actix;
extern crate actix_web;
extern crate futures;
extern crate dotenv;

use std::env;
use actix_web::{
    App, 
    AsyncResponder,
    Body,
    client,
    Error, 
    HttpMessage,
    HttpRequest, 
    HttpResponse, 
    server, 
    fs
};
// use actix_web::http::StatusCode;
use futures::{Future, Stream};
use dotenv::dotenv;

fn dev_proxy(_req: &HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let js_proxy = "http://localhost:8082/";
    client::ClientRequest::get(js_proxy)
        .finish().expect("failed to fetch js_proxy")
        .send()
        .map_err(Error::from)
        .and_then(|resp| {
            Ok(HttpResponse::Ok()
                .body(Body::Streaming(Box::new(resp.payload().from_err()))))
            
        })
        .responder()
}

fn dev_js_proxy(_req: &HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let js_proxy = "http://localhost:8082/app.js";
    client::ClientRequest::get(js_proxy)
        .finish().expect("failed to fetch js_proxy")
        .send()
        .map_err(Error::from)
        .and_then(|resp| {
            Ok(HttpResponse::Ok()
                .body(Body::Streaming(Box::new(resp.payload().from_err()))))
            
        })
        .responder()
}

fn main() {
    dotenv().ok();
    let host = "127.0.0.1:3000";
    let is_dev = env::var("ENV").expect("failed to read ENV") == "development";
    let sys = actix::System::new("upquark");
    let _addr = server::new(move || {
            let app = if is_dev {
                App::new()
                    .handler("/img", fs::StaticFiles::new("ui/dist/img").expect("fail to handle static images"))
                    .handler("/js", fs::StaticFiles::new("ui/dist/js").expect("fail to handle static js"))
                    .handler("/css", fs::StaticFiles::new("ui/dist/css").expect("fail to handle static css"))
                    .resource("/app.js", |r| r.f(dev_js_proxy))
                    .resource("/", |r| r.f(dev_proxy))
            } else {
                App::new()
                    .handler("/img", fs::StaticFiles::new("ui/dist/img").expect("fail to handle static images"))
                    .handler("/js", fs::StaticFiles::new("ui/dist/js").expect("fail to handle static js"))
                    .handler("/css", fs::StaticFiles::new("ui/dist/css").expect("fail to handle static css"))
                    .handler("/", fs::StaticFiles::new("ui/dist").expect("fail to handle static files").index_file("index.html"))
            };
            app
        })
    .bind(host).expect(&format!("could not bind to {}", host))
    .start();
        
    println!("Starting http server: {}", host);
    let _ = sys.run();
}