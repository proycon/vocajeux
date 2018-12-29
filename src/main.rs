extern crate clap;
extern crate rand;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate regex;
extern crate md5;
extern crate ansi_term;
extern crate dirs;

use std::fs;
use std::error::Error;
use std::fmt;
use std::iter::Iterator;
use std::io::{BufRead,Write};
use std::path::{Path,PathBuf};
use std::collections::HashMap;
use clap::{App, Arg, SubCommand};
use md5::{compute,Digest};
use regex::Regex;
use ansi_term::Colour::{Red,Green, Blue};

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

#[derive(Serialize, Deserialize)]
struct VocaScore {
    scores: HashMap<String,f64>,
    lastseen: HashMap<String,u64>
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

/// Load score file
fn load_scoredata(filename: &str) -> Result<VocaScore, Box<dyn Error>> {
    let data = fs::read_to_string(filename)?;
    let mut data: VocaScore = serde_json::from_str(data.as_str())?; //(shadowing)
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
fn select_item<'a>(data: &'a VocaList, mut optscoredata: Option<&mut VocaScore>) -> &'a VocaItem {
    let choice: f64 = rand::random::<f64>() * (data.items.len() as f64);
    let choice: usize = choice as usize;
    let id: String = format!("{:x}",data.items[choice].id());
    if let Some(ref mut scoredata) = optscoredata {
        scoredata.lastseen.insert(id,0);
    }
    &data.items[choice]
}

fn getinputline() -> Option<String> {
    print!(">>> ");
    std::io::stdout().flush().unwrap();
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

fn quizprompt(vocaitem: &VocaItem, phon: bool) {
    if phon {
        println!("{}: {} ({})", Blue.paint("Translate"), vocaitem, vocaitem.transcription);
    } else {
        println!("{}: {}", Blue.paint("Translate"), vocaitem);
    }
}

///Quiz
fn quiz(data: &VocaList, mut optscoredata: Option<&mut VocaScore>, phon: bool) {
    println!("QUIZ (type p for phonetic transcription, x for example, ENTER to skip)");
    let guesses = 3;
    loop {
        //select a random item
        let vocaitem;
        if let Some(ref mut scoredata) = optscoredata {
            vocaitem = select_item(data, Some(scoredata));
        } else {
            vocaitem = select_item(data, None);
        }
        quizprompt(vocaitem, phon);
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
                        println!("{}", Green.paint("Correct!"));
                        break;
                    }
                }
            } else {
                break;
            }
            println!("{} Try again (or ENTER to skip)", Red.paint("Incorrect!"));
        }
        if !correct {
            println!("The correct translation is: {}", Green.paint(&vocaitem.translation));
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
                let candidate  = select_item(data, None);
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
fn multiquiz(data: &VocaList, mut optscoredata: Option<&mut VocaScore>, choicecount: u32, phon: bool) {
    println!("MULTIPLE-CHOICE QUIZ (type p for phonetic transcription, x for example, ENTER to skip)");
    loop {
        //select a random item
        let vocaitem;
        if let Some(ref mut scoredata) = optscoredata {
            vocaitem = select_item(data, Some(scoredata));
        } else {
            vocaitem = select_item(data, None);
        }
        quizprompt(vocaitem, phon);
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
            } else {
                break;
            }
        }
        match correct {
            true => println!("{}", Green.paint("Correct!")),
            false => println!("{}; the correct translation is: {}", Red.paint("Incorrect"), Green.paint(&vocaitem.translation))
        }
        println!();
    }
}

/// Returns an index of available vocabulary sets
fn getdataindex() -> Vec<PathBuf> {
    let mut index: Vec<PathBuf> = Vec::new();
    let configpath = dirs::config_dir().unwrap();
    let datapath = PathBuf::from(configpath).join("vocajeux").join("data");
    if datapath.exists() {
        for file in datapath.read_dir().expect("Unable to read dir") {
            if let Ok(file) = file {
                index.push(file.path());
            }
        }
    }
    index
}

fn getdatafile(name: &str) -> Option<PathBuf> {
    let configpath = dirs::config_dir().unwrap();
    let mut filename: String = name.to_owned();
    filename.push_str(".json");
    let datapath = PathBuf::from(configpath).join("vocajeux").join("data").join(filename);
    match datapath.exists() {
        true => Some(datapath),
        false => None
    }
}

fn getscorefile(name: &str) -> Option<PathBuf> {
    let configpath = dirs::config_dir().unwrap();
    let mut filename: String = name.to_owned();
    filename.push_str(".json");
    let datapath = PathBuf::from(configpath).join("vocajeux").join("scores").join(filename);
    match datapath.exists() {
        true => Some(datapath),
        false => None
    }
}

fn main() {
    let argmatches = App::new("Vocajeux")
        .version("0.1")
        .author("Maarten van Gompel (proycon) <proycon@anaproy.nl>")
        .about("Games for learning vocabulary")
        .subcommand(SubCommand::with_name("catalogue")
                    .about("Lists all available datasets")
        )
        .subcommand(SubCommand::with_name("list")
                    .about("Lists all words")
                    .arg(Arg::with_name("file")
                        .help("Vocabulary file to load, either a full path or from in ~/.config/vocajeux/data/")
                        .index(1)
                        .required(true))
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
                    .arg(Arg::with_name("file")
                        .help("Vocabulary file to load, either a full path or from in ~/.config/vocajeux/data/")
                        .index(1)
                        .required(true))
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

    match argmatches.subcommand_name() {
        None => {
            eprintln!("No command given, see --help for syntax");
            std::process::exit(1);
        },
        Some("catalogue") =>  {
            let dataindex = getdataindex();
            for file in dataindex.iter() {
                println!("{}", file.to_str().unwrap());
            }
        },
        _ => { // all other subcommands that take a file parameter
            let submatches = argmatches.subcommand_matches(argmatches.subcommand_name().unwrap()).unwrap();
            let filename = submatches.value_of("file").expect("Expected filename");
            let mut datafile: Option<String> = None;
            if Path::new(filename).exists() {
                eprintln!("Loading {}", filename);
                datafile = Some(filename.to_string());
            } else {
                if let Some(founditem) = getdatafile(filename) {
                    //Option<PathBuf> to Option<String>
                    datafile = Some(founditem.to_str().unwrap().to_string());
                }
                //This would iterate over all available files but is unnecessarily expensive
                //compared to the above:
                /*if let Some(founditem) = dataindex.iter().find(|e| e.file_stem().unwrap() == filename) {
                    datafile = founditem.to_str();
                }*/
            }
            if datafile == None {
                eprintln!("Data file not found");
                std::process::exit(1);
            }
            let filebase = PathBuf::from(datafile.clone().unwrap().as_str());
            let scorefile = getscorefile(filebase.to_str().unwrap());

            match parse_vocadata(&datafile.unwrap()) {
                Ok(data) => {
                    //see what subcommand to perform
                    match argmatches.subcommand_name() {
                        Some("list") => {
                            list(&data, submatches.is_present("translations"), submatches.is_present("phon"));
                        },
                        Some("quiz") => {
                            let mut scoredata: Option<VocaScore> = load_scoredata(scorefile.unwrap().to_str().unwrap()).ok();
                            if submatches.is_present("multiplechoice") {
                                if let Some(choicecount) = submatches.value_of("multiplechoice") {
                                    let choicecount: u32 = choicecount.parse().unwrap();
                                    multiquiz(&data, scoredata.as_mut(), choicecount, submatches.is_present("phon"));
                                }
                            } else {
                                quiz(&data, scoredata.as_mut() , submatches.is_present("phon"));
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
}

