extern crate clap;
extern crate rand;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate regex;

use std::fs;
use std::error::Error;
use std::fmt;
use std::iter::Iterator;
use std::io::BufRead;
use clap::{App, Arg, SubCommand};
use md5::{compute,Digest};
use regex::Regex;

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

///Select a word
fn select_item(data: &VocaList) -> &VocaItem {
    let choice: f64 = rand::random::<f64>() * (data.items.len() as f64);
    let choice: usize = choice as usize;
    &data.items[choice]
}

fn getinputline() -> Option<String> {
    let stdin = std::io::stdin();
    let response = stdin.lock().lines().next().unwrap().unwrap(); //read one line only
    if response != "" {
        return Some(response);
    } else {
        return None;
    }
}


fn checktranslation(input: &String, reference: &String) -> bool {
    for candidate in  Regex::new(r"\b[\w\s]+\b").unwrap().find_iter(reference) {
        let candidate = candidate.as_str().to_lowercase();
        if candidate == input.to_lowercase() {
            return true;
        }
    }
    false
}

///Quiz
fn quiz(data: &VocaList, phon: bool) {
    println!("QUIZ (type p for phonetic transcription, x for example)");
    let guesses = 3;
    loop {
        //select a random item
        let vocaitem = select_item(data);
        if phon {
            println!("Translate: {} ({})", vocaitem, vocaitem.transcription);
        } else {
            println!("Translate: {}", vocaitem);
        }
        let mut correct = false;
        for _ in 0..guesses {
            //get response from user
            if let Some(response) = getinputline() {
                if response == "p" {
                    println!("{}", vocaitem.transcription);
                    continue;
                } else if response == "x" {
                    println!("{}", vocaitem.example);
                    continue;
                } else {
                    correct = checktranslation(&response, &vocaitem.translation);
                    if correct {
                        println!("Correct!");
                        break;
                    }
                }
            } else {
                break;
            }
            println!("Incorrect! Try again (or ENTER to skip)");
        }
        if !correct {
            println!("The correct translation is: {}", vocaitem.translation);
        }
        println!();
    }
}

fn getquizoptions<'a>(data: &'a VocaList, correctitem: &'a VocaItem, optioncount: u32) -> (Vec<&'a VocaItem>, u32) {
    //reserve an index for the correct option
    let correctindex: f64 = rand::random::<f64>() * (optioncount as f64);
    let correctindex: u32 = correctindex as u32;
    let mut options: Vec<&VocaItem> = Vec::new();
    for i in 0..optioncount {
        if i == correctindex {
            options.push(correctitem);
        } else {
            loop {
                let candidate = select_item(data);
                if candidate.id() != correctitem.id() {
                    options.push(candidate);
                    break;
                }
            }
        }
    }
    (options, correctindex)
}

///Multiple-choice Quiz
fn multiquiz(data: &VocaList, choicecount: u32, phon: bool) {
    println!("MULTIPLE-CHOICE QUIZ (type p for phonetic transcription, x for example)");
    loop {
        //select a random item
        let vocaitem = select_item(data);
        if phon {
            println!("Translate: {} ({})", vocaitem, vocaitem.transcription);
        } else {
            println!("Translate: {}", vocaitem);
        }
        let (options, correctindex) = getquizoptions(&data, &vocaitem, choicecount);
        for (i, option) in options.iter().enumerate() {
            println!("{} - {}", i+1, option.translation);
        }
        let mut correct = false;
        loop {
            //get response from user
            if let Some(response) = getinputline() {
                if response == "p" {
                    println!("{}", vocaitem.transcription);
                    continue;
                } else if response == "x" {
                    println!("{}", vocaitem.example);
                    continue;
                } else if let Ok(responseindex) = response.parse::<usize>() {
                    correct = responseindex -1 == correctindex as usize;
                    break;
                } else {
                    println!("Enter a number!");
                }
            }
        }
        match correct {
            true => println!("Correct!"),
            false => println!("Incorrect; the correct translation is: {}", vocaitem.translation)
        }
        println!();
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
        .subcommand(SubCommand::with_name("quiz")
                    .about("Simple quiz")
                    .arg(Arg::with_name("phon")
                         .help("Show phonetic transcription")
                         .long("phon")
                         .short("p")
                    )
                    .arg(Arg::with_name("multiplechoice")
                         .help("Multiple choice (number of choices)")
                         .long("multiplechoice")
                         .short("m")
                         .takes_value(true)
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
                    Some("quiz") => {
                        if let Some(submatches) = argmatches.subcommand_matches("quiz") {
                            if submatches.is_present("multiplechoice") {
                                if let Some(choicecount) = submatches.value_of("multiplechoice") {
                                    let choicecount: u32 = choicecount.parse().unwrap();
                                    multiquiz(&data, choicecount, submatches.is_present("phon"));
                                }
                            } else {
                                quiz(&data, submatches.is_present("phon"));
                            }
                        } else {
                            quiz(&data, false);
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

