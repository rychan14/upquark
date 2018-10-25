extern crate actix;
extern crate actix_web;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate futures;
// extern crate dotenv;
#[macro_use] extern crate juniper;

// use std::env;
use actix::prelude::*;
use actix_web::{
    App, 
    AsyncResponder,
    // Body,
    // client,
    Error, 
    FutureResponse,
    fs,
    http,
    // HttpMessage,
    HttpRequest, 
    HttpResponse, 
    Json,
    middleware,
    server, 
    State,
};
// use actix_web::http::StatusCode;
use fs::NamedFile;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use futures::future::{
    Future,
    // Stream,
};
// use dotenv::dotenv;

mod schema;

use schema::create_schema;
use schema::Schema;

struct AppState {
    executor: Addr<GraphQLExecutor>
}

pub struct GraphQLData(GraphQLRequest);

impl Message for GraphQLData {
    type Result = Result<String, Error>;
}

pub struct GraphQLExecutor {
    schema: std::sync::Arc<Schema>,
}

impl GraphQLExecutor {
    fn new(schema: std::sync::Arc<Schema>) -> GraphQLExecutor {
        GraphQLExecutor { schema: schema }
    }
}

impl Actor for GraphQLExecutor {
    type Context = SyncContext<Self>;
}

impl Handler<GraphQLData> for GraphQLExecutor {
    type Result = Result<String, Error>;

    fn handle(&mut self, msg: GraphQLData, _:&mut Self::Context) -> Self::Result {
        let res = msg.0.execute(&self.schema, &());
        let res_text = serde_json::to_string(&res)?;
        Ok(res_text)
    }
}

fn index(_req: &HttpRequest) -> Result<NamedFile, Error> {
    Ok(NamedFile::open("ui/dist/index.html")?)
}

fn graphiql(_req: &HttpRequest<AppState>,) -> Result<HttpResponse, Error> {
    let html = graphiql_source("http://127.0.0.1:3000/graphql");
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
    )
}

fn graphql((st, data): (State<AppState>, Json<GraphQLData>),) -> FutureResponse<HttpResponse> {
    st.executor
        .send(data.0)
        .from_err()
        .and_then(|res| match res {
            Ok(user) => Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(user)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}


fn main() {
    let host = "127.0.0.1:3000";
    // dotenv().ok();
    let sys = actix::System::new("upquark");
    let schema = std::sync::Arc::new(create_schema());
    let addr = SyncArbiter::start(3, move || GraphQLExecutor::new(schema.clone()));

    server::new(move || {
        vec![
            App::new()
                .resource("/", |r| r.f(index))
                .handler("/img", fs::StaticFiles::new("ui/dist/img").expect("fail to handle static images"))
                .handler("/js", fs::StaticFiles::new("ui/dist/js").expect("fail to handle static js"))
                .handler("/css", fs::StaticFiles::new("ui/dist/css").expect("fail to handle static css"))
                .handler("/", fs::StaticFiles::new("ui/dist").expect("fail to handle static files")),
            App::with_state(AppState{executor: addr.clone()})
                .middleware(middleware::Logger::default())
                .resource("/graphql", |r| r.method(http::Method::POST).with(graphql))
                .resource("/graphiql", |r| r.method(http::Method::GET).h(graphiql))
        ]
    }).bind(host).expect(&format!("could not bind to {}", host))
    .start();
        
    println!("Starting http server: {}", host);
    let _ = sys.run();
}