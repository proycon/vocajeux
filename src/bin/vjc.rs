extern crate clap;
extern crate reqwest;
extern crate serde;

use std::error::Error;
use std::process::exit;
use clap::{App, Arg, SubCommand};
use vocajeux::*;

#[derive(serde::Deserialize)]
struct Index {
    names: Vec<String>
}

fn index(url: &str) -> Result<Index, reqwest::Error> {
    let json: Index  = reqwest::get(url)?.json()?;
    Ok(json)
}

fn pick(url: &str, dataset: &str, accesskey: Option<&str>, seen: bool) -> Result<VocaItem, reqwest::Error> {
    let seen = match seen {
        true => "yes",
        false => "no",
    };
    let url = if let Some(accesskey) = accesskey {
        format!("{}/pick/{}/{}/?seen={}", url, dataset, accesskey, seen)
    } else {
        format!("{}/pick/{}/?seen={}", url, dataset, seen)
    };
    let json: VocaItem  = reqwest::get(url.as_str())?.json()?;
    Ok(json)
}

fn find(url: &str, dataset: &str, word: &str, accesskey: Option<&str>, seen: bool) -> Result<VocaItem, reqwest::Error> {
    let seen = match seen {
        true => "yes",
        false => "no",
    };
    let url = if let Some(accesskey) = accesskey {
        format!("{}/find/{}/{}/{}/?seen={}", url, dataset, word, accesskey, seen)
    } else {
        format!("{}/find/{}/{}/?seen={}", url, dataset, word, seen)
    };
    let json: VocaItem  = reqwest::get(url.as_str())?.json()?;
    Ok(json)
}

fn show(url: &str, dataset: &str) -> Result<VocaList, reqwest::Error> {
    let url = format!("{}/show/{}/", url, dataset);
    let json: VocaList  = reqwest::get(url.as_str())?.json()?;
    Ok(json)
}


fn main() {
    let mut success = true; //determines the exit code
    let arg_dataset = Arg::with_name("dataset")
                        .help("Name of the vocabulary set to load")
                        .long("dataset")
                        .short("s");
    let arg_tags = Arg::with_name("tags")
                        .help("Filter on tags, comma separated list")
                        .long("tags")
                        .short("T");
    let arg_phon = Arg::with_name("phon")
                         .help("Show phonetic transcription")
                         .long("phon")
                         .short("p");
    let arg_translations = Arg::with_name("translations")
                         .help("Show translations")
                         .long("translations")
                         .short("t");
    let arg_examples = Arg::with_name("examples")
                         .help("Show examples")
                         .long("examples")
                         .short("x");
    let arg_comments = Arg::with_name("comments")
                         .help("Show comments")
                         .long("comments")
                         .short("C");
    let argmatches = App::new("Vocajeux Client")
        .version("0.1")
        .author("Maarten van Gompel (proycon) <proycon@anaproy.nl>")
        .about("Games for learning vocabulary - client")
        .arg(Arg::with_name("url")
             .help("URL")
             .long("url")
             .takes_value(true)
             .short("u")
             .required(true)
        )
        .arg(Arg::with_name("accesskey")
             .help("Access key")
             .long("accesskey")
             .takes_value(true)
             .short("K")
             .required(true)
        )
        .arg(Arg::with_name("debug")
             .help("Debug")
             .long("debug")
             .short("D")
        )
        .subcommand(SubCommand::with_name("ls")
                    .about("Lists all available datasets")
        )
        .subcommand(SubCommand::with_name("show")
                    .about("Show the entire vocabulary list")
                    .arg(arg_dataset.clone())
                    .arg(arg_tags.clone())
                    .arg(arg_translations.clone())
                    .arg(arg_examples.clone())
                    .arg(arg_comments.clone())
                    .arg(Arg::with_name("showtags")
                         .help("Show tags")
                         .long("showtags")
                    )
                    .arg(arg_phon.clone()))
        .subcommand(SubCommand::with_name("flashcards")
                    .about("Flashcards")
                    .arg(arg_dataset.clone())
                    .arg(arg_tags.clone())
                    .arg(arg_phon.clone()))
        .subcommand(SubCommand::with_name("pick")
                    .about("Pick and display a random word")
                    .arg(arg_dataset.clone())
                    .arg(arg_tags.clone())
                    .arg(arg_phon.clone())
                    .arg(arg_translations.clone())
                    .arg(arg_examples.clone())
                    .arg(arg_comments.clone()))
        .subcommand(SubCommand::with_name("find")
                    .about("Find and display a specific word")
                    .arg(Arg::with_name("word")
                        .help("The word")
                        .index(2)
                        .required(true))
                    .arg(arg_dataset.clone())
                    .arg(arg_tags.clone())
                    .arg(arg_phon.clone())
                    .arg(arg_translations.clone())
                    .arg(arg_examples.clone())
                    .arg(arg_comments.clone()))
        .subcommand(SubCommand::with_name("quiz")
                    .about("Simple open quiz")
                    .arg(arg_dataset.clone())
                    .arg(arg_tags.clone())
                    .arg(arg_phon.clone()))
        .subcommand(SubCommand::with_name("choicequiz")
                    .about("Simple multiple-choice quiz")
                    .arg(arg_dataset.clone())
                    .arg(arg_tags.clone())
                    .arg(Arg::with_name("multiplechoice")
                         .help("Multiple choice (number of choices)")
                         .long("multiplechoice")
                         .short("m")
                         .takes_value(true)
                         .default_value("6")
                    )
                    .arg(arg_phon.clone()))
        .subcommand(SubCommand::with_name("matchquiz")
                    .arg(arg_dataset.clone())
                    .arg(arg_tags.clone())
                    .arg(Arg::with_name("number")
                         .help("Number of pairs to match")
                         .long("number")
                         .short("n")
                         .takes_value(true)
                         .default_value("6")
                    )
                    .arg(arg_phon.clone()))
        .get_matches();

    let debug = argmatches.is_present("debug");
    let url = argmatches.value_of("url").expect("no url specified");
    match argmatches.subcommand_name() {
        None => {
            eprintln!("No command given, see --help for syntax");
            std::process::exit(1);
        },
        Some(command) => {
            let submatches = argmatches.subcommand_matches(argmatches.subcommand_name().unwrap()).unwrap();
            match command {
                "ls" =>  {
                    match index(url) {
                        Ok(dataindex) => {
                            for name in dataindex.names.iter() {
                                println!("{}", name);
                            }
                        }
                        Err(err) => {
                            eprintln!("ERROR: {}", err);
                            success = false;
                        }
                    }
                },
                "pick" => {
                    match pick(url, argmatches.value_of("dataset").expect("No dataset specified"), argmatches.value_of("accesskey"), true) {
                        Ok(vocaitem) => vocaitem.print(submatches.is_present("phon"), submatches.is_present("translation"), submatches.is_present("example")),
                        Err(err) => println!("ERROR: {}", err),
                    }
                },
                "find" => {
                    match find(url, argmatches.value_of("dataset").expect("No dataset specified"), argmatches.value_of("word").expect("Word not provided") ,argmatches.value_of("accesskey"), true) {
                        Ok(vocaitem) => vocaitem.print(submatches.is_present("phon"), submatches.is_present("translation"), submatches.is_present("example")),
                        Err(err) => println!("ERROR: {}", err),
                    }
                },
                "show" => {
                    match show(url, argmatches.value_of("dataset").expect("No dataset specified")) {
                        Ok(vocalist) => {
                            vocalist.show(submatches.is_present("translation"), submatches.is_present("phon"), None, false, submatches.is_present("example"), false);
                        }
                        Err(err) => println!("ERROR: {}", err),
                    }
                },
                _ => {
                    eprintln!("Not implemented yet");
                }
            }
        }
    }
    exit(match success {
        true => 0,
        false => 1,
    });
}
