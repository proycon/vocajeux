extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate actix_web;
extern crate clap;

use actix_web::{server,http,App,HttpRequest,HttpResponse, Responder, Json};
use vocajeux::*;
use std::sync::Arc;

#[derive(Serialize)]
struct Index {
    catalogue: Vec<String>
}

#[derive(Clone)]
struct AppState {
    datadir: Arc<String>,
    scoredir: Arc<String>
}

fn index(_req: HttpRequest<AppState>) -> impl Responder {
    let dataindex = getdataindex(None);
    let index = Index { catalogue: dataindex.iter().map(|f| String::from(f.file_stem().unwrap().to_str().unwrap())).collect() };
    Json(index)
}

fn get(req: HttpRequest<AppState>) -> impl Responder {
    let dataset = req.match_info().get("dataset").unwrap();
    let query = req.query();
    if let Some(session) = query.get("session") {
       return HttpResponse::Ok().finish();
    } else {
       return HttpResponse::Unauthorized().finish();
    }
}

/*
fn app(state: AppState) -> App<AppState> {
    App::with_state(state)
            .resource("/", |res| res.method(http::Method::GET).with(index))
            .resource("/{dataset}/", |res| res.method(http::Method::GET).with(get))
}
*/

fn main() {
    let defaultdatadir = defaultdatadir();
    let defaultscoredir = defaultscoredir();

    let argmatches = clap::App::new("Vocajeuxws")
        .version("0.1")
        .author("Maarten van Gompel (proycon) <proycon@anaproy.nl>")
        .about("Vocabulary webservice")
        .arg(clap::Arg::with_name("datadir")
            .help("Data directory (default is ~/.config/vocajeux/data/")
            .short("d")
            .long("dir")
            .takes_value(true)
            .default_value(defaultdatadir.to_str().unwrap())
        )
        .arg(clap::Arg::with_name("host")
            .help("Host and port to bind to")
            .short("b")
            .long("bind")
            .takes_value(true)
            .default_value("127.0.0.1:8888")
        )
        .get_matches();

    let state = AppState {
                    datadir: Arc::new(argmatches.value_of("datadir").unwrap().to_string()),
                    scoredir: Arc::new(argmatches.value_of("scoredir").unwrap().to_string())
                };

    server::new(move || {
            App::with_state(state.clone())
                    .resource("/", |res| res.method(http::Method::GET).with(index))
                    .resource("/{dataset}/", |res| res.method(http::Method::GET).with(get))
        })
        .bind(argmatches.value_of("bind").expect("Host and port"))
        .unwrap()
        .run();
}
