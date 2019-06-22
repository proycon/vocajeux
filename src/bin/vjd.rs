extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate actix_web;
extern crate clap;

use actix_web::{server,http,App,HttpRequest,HttpResponse, Responder, Json};
use vocajeux::*;
use std::sync::Arc;
use std::error::Error;
use std::fmt;

#[derive(Serialize)]
struct Index {
    names: Vec<String>
}

#[derive(Clone)]
struct AppState {
    datadir: Arc<String>,
    scoredir: Arc<String>
}


#[derive(Debug, Clone, PartialEq, Eq)]
struct NotFoundError;

impl fmt::Display for NotFoundError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(self.description())
    }
}

impl Error for NotFoundError {
    fn description(&self) -> &str {
        "not found"
    }
}


fn index(_req: HttpRequest<AppState>) -> impl Responder {
    let dataindex = getdataindex(None);
    let index = Index {
        names: dataindex.iter().map( |f| String::from(f.file_stem().unwrap().to_str().unwrap()) ).collect()
    };
    Json(index)
}

/// Show the entire vocabulary list
fn show(req: HttpRequest<AppState>) -> impl Responder {
    if let Some(dataset) = req.match_info().get_decoded("dataset") {
        match loadvocalist(&dataset) {
            Ok(data) => {
                Json(data).respond_to(&req).unwrap_or(HttpResponse::NotFound().finish())
            },
            Err(err) => {
                HttpResponse::NotFound().body(format!("Not found: {}",err))
            }
        }
    } else {
        HttpResponse::NotFound().finish()
    }
}

fn loadvocalist(name: &str) -> Result<VocaList, Box<dyn Error> > {
   if let Some(datafile) = getdatafile(name, defaultdatadir()) {
        VocaList::parse(datafile.to_str().unwrap())
    } else {
        Err(NotFoundError.into())
    }
}


/// Pick a random word from a dataset
/*
fn pick(req: HttpRequest<AppState>) -> impl Responder {
    if Some(dataset) = req.match_info().get_decoded("dataset") {
        match loadvocalist(&dataset) {
            Ok(data) => {
              Json(data)
            },
            Err(_msg) => { //TODO: propagate _msg
              HttpResponse::NotFound().finish();
            }
        }
    } else {
        HttpResponse::NotFound().finish();
    }
}
*/

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
        .arg(clap::Arg::with_name("scoredir")
            .help("Score directory (default is ~/.config/vocajeux/scores/")
            .short("s")
            .long("scoredir")
            .takes_value(true)
            .default_value(defaultscoredir.to_str().unwrap())
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
                    .resource("/show/{dataset}/", |res| res.method(http::Method::GET).with(show))
                    //.resource("/pick/{dataset}/{session}", |res| res.method(http::Method::GET).with(pick))
        })
        .bind(argmatches.value_of("bind").expect("Host and port"))
        .unwrap()
        .run();
}
