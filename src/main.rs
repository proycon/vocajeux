extern crate clap;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::fs;
use std::error::Error;
use std::fmt;
use std::collections::HashMap;
use clap::{App, Arg, SubCommand};
use md5::{compute,Digest};

/// Vocabulary Item data structure
#[derive(Serialize, Deserialize)]
struct VocaItem {
    word: String,
    transcription: String,
    translation: String,
    example: String,
    comment: String,
    tags: Vec<String>
}

/// Vocabulary List data structure
#[derive(Serialize, Deserialize)]
struct VocaList {
    items: Vec<VocaItem>
}

//we implement the Display trait so we can print VocaItems
impl fmt::Display for VocaItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.word)
    }
}

impl VocaItem {
    fn id(&self) -> md5::Digest {
        md5::compute(self.word.as_bytes())
    }
}



/// Parse the vocabulary data file (JSON) into the VocaList structure
fn parse_vocadata(filename: &str) -> Result<VocaList, Box<dyn Error>> {
    let data = fs::read_to_string(filename)?;
    let data: VocaList = serde_json::from_str(data.as_str())?; //(shadowing)
    Ok(data)
}

/// List/Print the contents of the Vocabulary List to standard output
fn list(data: &VocaList, withtranslation: bool, withtranscription: bool) {
    for item in data.items.iter() {
        print!("{}", item);
        if withtranscription { print!("\t{}", item.transcription) }
        if withtranslation { print!("\t{}", item.translation) }
        println!()
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
                //see what subcommand to perform
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

