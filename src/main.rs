extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate futures;
// extern crate dotenv;
#[macro_use] 
extern crate juniper;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

// use std::env;
use actix::prelude::*;
use actix_web::{
    fs,
    http,
    middleware,
    server, 
    App, 
    AsyncResponder,
    Error, 
    FutureResponse,
    HttpRequest, 
    HttpResponse, 
    Json,
    State,
};
use futures::future::Future;
use fs::NamedFile;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;

mod schema;

use schema::create_schema;
use schema::Schema;

struct AppState {
    executor: Addr<GraphQLExecutor>,
}

#[derive(Serialize, Deserialize)]
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

fn graphiql(_req: &HttpRequest<AppState>) -> Result<HttpResponse, Error> {
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

fn app() -> App<> {
    App::new()
        .resource("/", |r| r.f(index))
        .handler("/img", fs::StaticFiles::new("ui/dist/img").expect("fail to handle static images"))
        .handler("/js", fs::StaticFiles::new("ui/dist/js").expect("fail to handle static js"))
        .handler("/css", fs::StaticFiles::new("ui/dist/css").expect("fail to handle static css"))
        .handler("/", fs::StaticFiles::new("ui/dist").expect("fail to handle static files"))

}

fn graphql_app() -> App<AppState> {
    let schema = std::sync::Arc::new(create_schema());
    let addr = SyncArbiter::start(3, move || GraphQLExecutor::new(schema.clone()));
    App::with_state(AppState{executor: addr.clone()})
        .prefix("api")
        .middleware(middleware::Logger::default())
        .resource("/graphql", |r| r.method(http::Method::POST).with(graphql))
        .resource("/graphiql", |r| r.method(http::Method::GET).h(graphiql))

}

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let host = "localhost:3000";
    let sys = actix::System::new("upquark");
    server::new(move || {
        vec![
            graphql_app().boxed(),
            app().boxed(),
        ]
    }).bind(host)
        .expect(&format!("could not bind to {}", host))
        .start();
        
    println!("Starting http server: {}", host);
    let _ = sys.run();
}