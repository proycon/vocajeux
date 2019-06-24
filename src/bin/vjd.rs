extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate actix_web;
extern crate clap;

use vocajeux::*;
use actix_web::{server,http,App,HttpRequest,HttpResponse, Responder, Json};
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::{Arc,Mutex,RwLock};
use std::error::Error;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::thread;

#[derive(Serialize)]
struct Index {
    names: Vec<String>
}

#[derive(Clone)]
struct AppState {
    datadir: Arc<String>,
    scoredir: Arc<String>,
    data: Arc<RwLock<HashMap<String,VocaList>>>, //RwLock allows multiple read locks at the same time, Mutex doesn't distinguish between reading and writing and lock for all
    scores: Arc<Mutex<HashMap<(String,String),VocaScore>>>,
    data_lastused: Arc<Mutex<HashMap<String,u64>>>,
    scores_lastused: Arc<Mutex<HashMap<(String,String),u64>>>
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


///Adds a vocabulary list to the loaded data
fn addvocalist<'a>(state: &'a AppState, dataset: &'a str) -> Result<(), Box<(dyn Error + 'static)> > {
    let mut vocalists = state.data.write().expect("RwLock poisoned");
    if !vocalists.contains_key(dataset) {
        let vocalist = loadvocalist(state, dataset)?;
        vocalists.insert(dataset.to_string(), vocalist);
    }
    let mut lastused = state.data_lastused.lock().expect("Lock failed");
    let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Unable to get time").as_secs();
    lastused.insert(dataset.to_string(), now);
    Ok(())
}

///Adds a vocabulary score to the loaded data
fn addvocascore<'a>(state: &'a AppState, dataset: &'a str, sessionkey: &'a str) -> Result<(), Box<(dyn Error + 'static)> > {
    let mut scores = state.scores.lock().expect("Unable to lock");
    let scorekey = (dataset.to_string(), sessionkey.to_string());
    if !scores.contains_key(&scorekey) {
        let scoremap = loadvocascore(state, dataset, sessionkey)?;
        scores.insert(scorekey.clone(), scoremap);
    }
    let mut lastused = state.scores_lastused.lock().expect("Lock failed");
    let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Unable to get time").as_secs();
    lastused.insert(scorekey, now);
    Ok(())
}

///Loads and returns a vocabulary list
fn loadvocalist(state: &AppState, dataset: &str) -> Result<VocaList, Box<dyn Error> > {
   let datadir = &*state.datadir; //deref arc and borrow
   if let Some(datafile) = getdatafile(dataset, PathBuf::from(datadir)) {
        VocaList::parse(datafile.to_str().unwrap())
    } else {
        Err(NotFoundError.into()) //into box
    }
}

///Loads and returns a vocabulary scoremap
fn loadvocascore(state: &AppState, dataset: &str, sessionkey: &str) -> Result<VocaScore,Box<dyn Error> > {
    let scoredir = &*state.scoredir; //deref arc and borrow
    let scorefile = getscorefile(dataset, PathBuf::from(scoredir), Some(sessionkey));
    if scorefile.exists() {
       VocaScore::load(scorefile.to_str().unwrap())
    } else {
        Ok(VocaScore::default()) //a new one
    }
}

fn cleanup(state: &AppState) {
    let expiration = 900; //15 minutes
    let mut scores = state.scores.lock().expect("Unable to get score lock");
    let mut scores_lastused = state.scores_lastused.lock().expect("Unable to get score lastused lock");
    let mut scores_expire: Vec<(String,String)> = Vec::new();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Unable to get time").as_secs();
    for (scorekey, lastused) in scores_lastused.iter() {
        if now - lastused >= expiration {
            scores_expire.push(scorekey.clone());
        }
    }
    for scorekey in scores_expire.iter() {
        scores_lastused.remove(scorekey);
        scores.remove(scorekey);
    }
    let mut data = state.data.write().expect("Unable to get data lock");
    let mut data_lastused = state.data_lastused.lock().expect("Unable to get data lock");
    let mut data_expire: Vec<String> = Vec::new();
    for (dataset, lastused) in data_lastused.iter() {
        if now - lastused >= expiration {
            data_expire.push(dataset.clone());
        }
    }
    for dataset in data_expire.iter() {
        data_lastused.remove(dataset);
        data.remove(dataset);
    }
}

// REST API endpoints

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
        match loadvocalist(&req.state(), &dataset) { //loads directly from file rather than using the one in the state
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


///Generic handle function that takes care of loading vocabulary lists and score lists and finally
///passes control to another specified function
fn handle(req: HttpRequest<AppState>, handler: impl FnOnce(&HttpRequest<AppState>, &VocaList,Option<&mut VocaScore>, bool) -> HttpResponse) -> impl Responder {
    let state = &req.state();
    //parse query parameter:
    let seen = match req.query().get("seen").map(|x| { x.as_str() }) {
        Some("no") | Some("0") | Some("false") => true,
        _ => false
    };

    if let Some(dataset) = req.match_info().get_decoded("dataset"){
        match addvocalist(state, &dataset) {
            Ok(_) => {
                let mut scores = state.scores.lock().expect("Unable to get score lock");
                let sessionkey = req.match_info().get_decoded("session");
                let vocascore = if let Some(sessionkey) = sessionkey {
                    addvocascore(state,&dataset,&sessionkey).ok();
                    let scorekey = (dataset.to_string(), sessionkey.to_string());
                    scores.get_mut(&scorekey)
                } else {
                    None
                };

                let vocalists = state.data.read().expect("Unable to get data lock");

                match vocalists.get(&dataset) {
                    Some(vocalist) => {
                        handler(&req, vocalist, vocascore, seen)
                    },
                    None => {
                        HttpResponse::NotFound().body("Unable to retrieve loaded vocabulary list")
                    }
                }
            }
            Err(err) => {
                HttpResponse::NotFound().body(format!("Not found: {}",err))
            }
        }
    } else {
        HttpResponse::NotFound().finish()
    }
}


///Get a random item from a vocabulary list
fn pick(req: HttpRequest<AppState>) -> impl Responder {
    handle(req, |req,vocalist, vocascore, seen| {
        let vocaitem = vocalist.pick(vocascore,None, seen);
        Json(vocaitem).respond_to(&req).unwrap_or(HttpResponse::NotFound().finish())
    })
}

///Get a specific item from a vocabulary list
fn find(req: HttpRequest<AppState>) -> impl Responder {
    handle(req, |req,vocalist, vocascore, seen| {
        if let Some(word) = req.match_info().get("word") {
            if let Some(vocaitem) = vocalist.find(word, vocascore, seen) {
                Json(vocaitem).respond_to(&req).unwrap_or(HttpResponse::NotFound().finish())
            } else {
                HttpResponse::NotFound().body("Word not found")
            }
        } else {
            HttpResponse::NotFound().body("Word not specified")
        }
    })
}

///Mark an item as correct
fn score(req: HttpRequest<AppState>) -> impl Responder {
    handle(req, |req,vocalist, vocascore, _| {
        if let Some(vocascore) = vocascore {
            if let Some(word) = req.match_info().get("word") {
                if let Some(vocaitem) = vocalist.find(word, Some(vocascore), true) {
                    let correct: bool;
                    match req.query().get("correct").map(|x| { x.as_str() }) {
                        Some("yes") | Some("1") | Some("true") => { correct = true; },
                        Some("no") | Some("0") | Some("false") => { correct = false; },
                        Some(_) => { return HttpResponse::NotFound().body("Expected parameter 'correct' has invalid value"); }
                        None => { return HttpResponse::NotFound().body("Expected parameter 'correct' not found"); }
                    };
                    vocascore.addscore(vocaitem, correct);
                    HttpResponse::Ok()
                        .header(http::header::CONTENT_TYPE, http::header::ContentType::json())
                        .body("{}") //empty json response
                } else {
                    HttpResponse::NotFound().body("Word not found")
                }
            } else {
                HttpResponse::NotFound().body("Word not specified")
            }
        } else {
            HttpResponse::NotFound().body("No session specified")
        }
    })
}


fn main() {
    let defaultdatadir = defaultdatadir();
    let defaultscoredir = defaultscoredir();

    let argmatches = clap::App::new("vjd")
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
        .arg(clap::Arg::with_name("bind")
            .help("Host and port to bind to")
            .short("b")
            .long("bind")
            .takes_value(true)
            .default_value("127.0.0.1:8888")
        )
        .get_matches();


    let state = AppState {
                    datadir: Arc::new(argmatches.value_of("datadir").unwrap().to_string()),
                    scoredir: Arc::new(argmatches.value_of("scoredir").unwrap().to_string()),
                    data: Arc::new(RwLock::new(HashMap::new())),
                    scores: Arc::new(Mutex::new(HashMap::new())),
                    data_lastused: Arc::new(Mutex::new(HashMap::new())),
                    scores_lastused: Arc::new(Mutex::new(HashMap::new()))
                };

    let clonedstate = state.clone();

    thread::spawn(move || {
        loop {
            cleanup(&clonedstate);
            thread::sleep(Duration::from_secs(300)); //wait 5 minutes
        }
    });

    server::new(move || {
            App::with_state(state.clone())
                    .resource("/", |res| res.method(http::Method::GET).with(index))
                    .resource("/show/{dataset}/", |res| res.method(http::Method::GET).with(show))
                    .resource("/pick/{dataset}/", |res| res.method(http::Method::GET).with(pick))
                    .resource("/pick/{dataset}/{session}/", |res| res.method(http::Method::GET).with(pick))
                    .resource("/find/{dataset}/{word}/", |res| res.method(http::Method::GET).with(find))
                    .resource("/find/{dataset}/{word}/{session}/", |res| res.method(http::Method::GET).with(find))
                    .resource("/score/{dataset}/{word}/{session}/", |res| res.method(http::Method::GET).with(score))
        })
        .bind(argmatches.value_of("bind").expect("Host and port not provided or invalid"))
        .unwrap()
        .run();
}
