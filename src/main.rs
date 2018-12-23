extern crate clap;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::fs;
use std::error::Error;
use std::fmt;
use clap::{App, Arg, SubCommand};

#[derive(Serialize, Deserialize)]
struct VocaItem {
    word: String,
    transcription: String,
    translation: String,
    example: String,
    comment: String,
    tags: Vec<String>
}

impl fmt::Display for VocaItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.word)
    }
}

#[derive(Serialize, Deserialize)]
struct VocaList {
    items: Vec<VocaItem>
}


fn parse_vocadata(filename: &str) -> Result<VocaList, Box<dyn Error>> {
    let data = fs::read_to_string(filename)?;
    let data: VocaList = serde_json::from_str(data.as_str())?; //(shadowing)
    Ok(data)
}

fn list(data: &VocaList, withtranslation: bool) {
    for item in data.items.iter() {
        if withtranslation {
            println!("{}\t{}", item.word, item.translation)
        } else {
            println!("{}", item) //(rely on the Display trait)
        }
    }
}

fn main() {
    let argmatches = App::new("Vocajeux")
        .version("0.1")
        .author("Maarten van Gompel (proycon) <proycon@anaproy.nl>")
        .about("Games for learning vocabulary")
        .arg(Arg::with_name("file")
            .help("Vocabulary file to load")
            .index(1)
            .required(true))
        .subcommand(SubCommand::with_name("list")
                    .about("Lists all words")
                    .arg(Arg::with_name("translations")
                         .help("Show translations")
                         .long("translation")
                         .short("t")
                    ))
        .get_matches();

    if let Some(filename) = argmatches.value_of("file") {
        eprintln!("Loading {}", filename);
        match parse_vocadata(filename) {
            Ok(data) => {
                match argmatches.subcommand_name() {
                    Some("list") => {
                        if let Some(submatches) = argmatches.subcommand_matches("list") {
                            list(&data, submatches.is_present("translations"));
                        } else {
                            list(&data, false);
                        }
                    },
                    _ => {
                        eprintln!("Nothing to do!");
                    }
                }
            },
            Err(err) => {
                eprintln!("Error: {}", err);
            }
        }
    }
}
