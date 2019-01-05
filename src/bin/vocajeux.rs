extern crate clap;
extern crate rand;
extern crate serde;
extern crate regex;
extern crate ansi_term;
extern crate dirs;

use std::iter::Iterator;
use std::io::{BufRead,Write};
use std::path::{Path,PathBuf};
use clap::{App, Arg, SubCommand};
use regex::Regex;
use rand::{thread_rng,Rng};
use ansi_term::Colour::{Red,Green, Blue};
use vocajeux::*;


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
    println!("QUIZ (type p for phonetic transcription, x for example, q to quit, ENTER to skip)");
    let guesses = 3;
    loop {
        //select a random item
        let vocaitem;
        if let Some(ref scoredata) = optscoredata {
            vocaitem = data.pick(Some(scoredata));
        } else {
            vocaitem = data.pick(None);
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
                } else if response == "q" {
                    return;
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
        if let Some(ref mut scoredata) = optscoredata {
            scoredata.addscore(&vocaitem, correct);
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
                let candidate  = data.pick(None);
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
    println!("MULTIPLE-CHOICE QUIZ (type p for phonetic transcription, x for example, q to quit, ENTER to skip)");
    loop {
        //select a random item
        let vocaitem;
        if let Some(ref scoredata) = optscoredata {
            vocaitem = data.pick(Some(scoredata));
        } else {
            vocaitem = data.pick(None);
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
                } else if response == "q" {
                    return;
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
        if let Some(ref mut scoredata) = optscoredata {
            scoredata.addscore(&vocaitem, correct);
        }
        println!();
    }
}

fn parsematchresponse(vocaitems: &Vec<&VocaItem>, mappings: &Vec<u8>, response: String, optscoredata: &mut Option<&mut VocaScore>, solved: &mut Vec<u8>) -> bool {
    let bytes: Vec<u8> = response.into_bytes();
    if let (Some(first), Some(second)) = (bytes.get(0), bytes.get(1)) {
        let firstchar: char = *first as char;
        let secondchar: char = *second as char;
        if firstchar.is_ascii_digit() {
            if secondchar.is_ascii_alphabetic() {
                if let Some(mapped) = mappings.get(*first as usize - 1) {
                    let correct: bool = *mapped == (*second - 0x61u8);
                    if correct {
                        solved.push(*first);
                        println!("{}", Green.paint("Correct!"));
                    } else {
                        println!("{}", Red.paint("Wrong!"));
                    }
                    if let Some(ref mut scoredata) = optscoredata {
                        if let Some(vocaitem) = vocaitems.get(*first as usize - 1) {
                            scoredata.addscore(vocaitem, correct);
                        }
                    }
                    return true;
                }
            } else {
                println!("Expected a letter in the second position (for example: 1a)");
            }
        } else {
            println!("Expected a digit in the first position (for example: 1a)");
        }
    }
    false
}

///Match quiz
fn matchquiz(data: &VocaList, mut optscoredata: Option<&mut VocaScore>, matchcount: u8, phon: bool) {
    println!("MATCH QUIZ (Enter a match by entering a number and a letter, enter q to quit, s to skip)");
    loop {
        let mut vocaitems: Vec<&VocaItem> = Vec::new();
        for _i in 0..matchcount {
            let vocaitem;
            if let Some(ref scoredata) = optscoredata {
                vocaitem = data.pick(Some(scoredata));
            } else {
                vocaitem = data.pick(None);
            }
            vocaitems.push(vocaitem);
        }
        //create a random order for presentation of the translations
        let mut mappings: Vec<u8> = (0..matchcount).collect();
        thread_rng().shuffle(&mut mappings);

        for (i, vocaitem) in vocaitems.iter().enumerate() {
            if phon {
                println!("{}) {} ({})", i+1, vocaitem.word, vocaitem.transcription);
            } else {
                println!("{}) {}", i+1, vocaitem.word);
            }
        }
        println!("---match with:---");
        for (i, mappedindex) in mappings.iter().enumerate() {
            if let Some(vocaitem) = vocaitems.get(*mappedindex as usize) {
                let c: char = (0x61u8 + i as u8) as char;
                println!("{}) {}", c, vocaitem.translation);
            }
        }
        let mut solved: Vec<u8> = Vec::new();
        loop {
            //get response from user
            if let Some(response) = getinputline() {
                if response == "q" {
                    return;
                } else {
                    if parsematchresponse(&vocaitems, &mappings, response, &mut optscoredata, &mut solved) {

                    }
                }
            } else {
                break;
            }
        }
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

fn getscorefile(name: &str) -> PathBuf {
    let configpath = dirs::config_dir().unwrap();
    let mut filename: String = name.to_owned();
    filename.push_str(".json");
    PathBuf::from(configpath).join("vocajeux").join("scores").join(filename)
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
                        .help("Vocabulary file to load, either a full path or from ~/.config/vocajeux/data/")
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
                    .about("Simple open quiz")
                    .arg(Arg::with_name("file")
                        .help("Vocabulary file to load, either a full path or from ~/.config/vocajeux/data/")
                        .index(1)
                        .required(true))
                    .arg(Arg::with_name("phon")
                         .help("Show phonetic transcription")
                         .long("phon")
                         .short("p")
                    ))
        .subcommand(SubCommand::with_name("choicequiz")
                    .about("Simple multiple-choice quiz")
                    .arg(Arg::with_name("file")
                        .help("Vocabulary file to load, either a full path or from ~/.config/vocajeux/data/")
                        .index(1)
                        .required(true))
                    .arg(Arg::with_name("multiplechoice")
                         .help("Multiple choice (number of choices)")
                         .long("multiplechoice")
                         .short("m")
                         .takes_value(true)
                         .default_value("6")
                    )
                    .arg(Arg::with_name("phon")
                         .help("Show phonetic transcription")
                         .long("phon")
                         .short("p")
                    ))
        .subcommand(SubCommand::with_name("matchquiz")
                    .arg(Arg::with_name("file")
                        .help("Vocabulary file to load, either a full path or from ~/.config/vocajeux/data/")
                        .index(1)
                        .required(true))
                    .arg(Arg::with_name("number")
                         .help("Number of pairs to match")
                         .long("number")
                         .short("n")
                         .takes_value(true)
                         .default_value("6")
                    )
                    .arg(Arg::with_name("phon")
                         .help("Show phonetic transcription")
                         .long("phon")
                         .short("p")
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

            match VocaList::parse(&datafile.unwrap()) {
                Ok(data) => {
                    //see what subcommand to perform
                    match argmatches.subcommand_name() {
                        Some("list") => {
                            data.list(submatches.is_present("translations"), submatches.is_present("phon"));
                        },
                        Some("quiz") | Some("choicequiz") | Some("matchquiz") => {
                            let mut optscoredata: Option<VocaScore> = match scorefile.exists() {
                                true => VocaScore::load(scorefile.to_str().unwrap()).ok(),
                                false => Some(VocaScore { ..Default::default() } ),
                            };
                            match argmatches.subcommand_name() {
                                Some("choicequiz") => {
                                    if let Some(choicecount) = submatches.value_of("multiplechoice") {
                                        let choicecount: u32 = choicecount.parse().unwrap();
                                        multiquiz(&data, optscoredata.as_mut(), choicecount, submatches.is_present("phon"));
                                    }
                                },
                                Some("matchquiz") => {
                                    if let Some(matchcount) = submatches.value_of("number") {
                                        let matchcount: u8 = matchcount.parse().unwrap();
                                        matchquiz(&data, optscoredata.as_mut(), matchcount, submatches.is_present("phon"));
                                    }
                                },
                                Some("quiz") => {
                                    quiz(&data, optscoredata.as_mut() , submatches.is_present("phon"));
                                },
                                _ => {}
                            }
                            if let Some(ref scoredata) = optscoredata {
                                scoredata.save(scorefile.to_str().unwrap()).expect("Unable to save");
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
