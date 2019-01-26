extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate actix_web;

use actix_web::{server,http,App,HttpRequest, Responder, Json};
use vocajeux::*;

#[derive(Serialize)]
struct Index {
    catalogue: Vec<String>
}

fn index(_req: HttpRequest) -> impl Responder {
    let dataindex = getdataindex(None);
    let index = Index { catalogue: dataindex.iter().map(|f| String::from(f.to_str().unwrap())).collect() };
    Json(index)
}

fn app() -> App {
    App::new()
        .resource("/", |req| req.method(http::Method::GET).with(index))
}

fn main() {

}
