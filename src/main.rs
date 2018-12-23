extern crate clap;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::fs;
use std::error::Error;
use std::fmt;
use clap::{App, Arg, SubCommand};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize, Hash)]
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

fn list(data: &VocaList, withtranslation: bool, withtranscription: bool) {
    for item in data.items.iter() {
        print!("{}", item);
        if withtranscription { print!("\t{}", item.transcription) }
        if withtranslation { print!("\t{}", item.translation) }
        println!()
    }
}

fn hash<T: Hash>(t: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
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
                    )
                    .arg(Arg::with_name("phon")
                         .help("Show phonetic transcription")
                         .long("phon")
                         .short("p")
                    ))
        .get_matches();

    if let Some(filename) = argmatches.value_of("file") {
        eprintln!("Loading {}", filename);
        match parse_vocadata(filename) {
            Ok(data) => {
                match argmatches.subcommand_name() {
                    Some("list") => {
                        if let Some(submatches) = argmatches.subcommand_matches("list") {
                            list(&data, submatches.is_present("translations"), submatches.is_present("phon"));
                        } else {
                            list(&data, false, false);
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

