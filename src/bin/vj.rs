extern crate clap;
extern crate rand;
extern crate serde;
extern crate regex;
extern crate ansi_term;
extern crate dirs;

use std::iter::Iterator;
use std::io::{BufRead,Write};
use std::path::{Path,PathBuf};
use std::fs;
use clap::{App, Arg, SubCommand};
use regex::Regex;
use rand::{thread_rng,Rng};
use ansi_term::Colour::{Red,Green, Blue};
use vocajeux::*;

///Flashcards
fn flashcards(data: &VocaList, mut optscoredata: Option<&mut VocaScore>, phon: bool, filtertags: Option<&Vec<&str>>) {
    let instructions = "type ENTER to turn, q to quit, k for correct, i for incorrect";
    println!("FLASHCARDS ({})", instructions);
    println!("---------------------------------------------------------------------------------------");
    loop {
        //select a random item
        let vocaitem;
        if let Some(ref mut scoredata) = optscoredata {
            vocaitem = data.pick(Some(scoredata), filtertags, true);
        } else {
            vocaitem = data.pick(None, filtertags, true);
        }
        let mut turned = false;
        let correct;
        loop{
            if turned {
                println!("{}", vocaitem.transcription);
                println!("{}", vocaitem.translation);
                println!("{}", vocaitem.example);
            } else {
                quizprompt(vocaitem, phon);
                println!("{}", vocaitem.example);
            }
            //get response from user
            if let Some(response) = getinputline() {
                if response == "i" {
                    correct = false;
                    break;
                } else if response == "k" {
                    correct = true;
                    break;
                } else if response == "h" {
                    println!("{}",instructions);
                } else if response == "q" {
                    return;
                } else {
                    println!("{}", Red.paint("Invalid input"));
                }
            } else {
                turned = !turned;
            }
        }
        if let Some(ref mut scoredata) = optscoredata {
            scoredata.addscore(&vocaitem, correct);
        }
        println!();
    }
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

///Picks and prints a random item, provides no further interaction
fn pick(data: &VocaList, mut optscoredata: Option<&mut VocaScore>, phon: bool, translation: bool, example: bool, filtertags: Option<&Vec<&str>>) {
    //select a random item
    let vocaitem;
    if let Some(ref mut scoredata) = optscoredata {
        vocaitem = data.pick(Some(scoredata),filtertags,true);
    } else {
        vocaitem = data.pick(None,filtertags,true);
    }
    print(vocaitem, phon, translation, example);
}

///Looks up and prints a specific item, provides no further interaction
fn find(data: &VocaList, word: &str, mut optscoredata: Option<&mut VocaScore>, phon: bool, translation: bool, example: bool) {
    //select a random item
    let vocaitem;
    if let Some(ref mut scoredata) = optscoredata {
        vocaitem = data.find(word, Some(scoredata),true);
    } else {
        vocaitem = data.find(word, None,true);
    }
    if let Some(vocaitem) = vocaitem {
        print(vocaitem, phon, translation, example);
    } else {
        eprintln!("Not found");
    }
}

///Prints a vocaitem
fn print(vocaitem: &VocaItem, phon: bool, translation: bool, example: bool) {
    println!("{}", vocaitem.word);
    if phon {
        println!("{}", vocaitem.transcription);
    }
    if example {
        println!("{}", vocaitem.example);
    }
    if translation {
        println!("{}", vocaitem.translation);
    }
}


///Quiz
fn quiz(data: &VocaList, mut optscoredata: Option<&mut VocaScore>, phon: bool, filtertags: Option<&Vec<&str>>) {
    let instructions = "type p for phonetic transcription, x for example, q to quit, ENTER to skip";
    println!("QUIZ ({})", instructions);
    println!("---------------------------------------------------------------------------------");
    let guesses = 3;
    loop {
        //select a random item
        let vocaitem;
        if let Some(ref mut scoredata) = optscoredata {
            vocaitem = data.pick(Some(scoredata),filtertags,true);
        } else {
            vocaitem = data.pick(None,filtertags,true);
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
                } else if response == "h" {
                    println!("{}",instructions);
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
        if let Some(ref mut scoredata) = optscoredata {
            scoredata.addscore(&vocaitem, correct);
        }
        if !correct {
            println!("The correct translation is: {}", Green.paint(&vocaitem.translation));
        }
        println!();
    }
}

fn getquizoptions<'a>(data: &'a VocaList, correctitem: &'a VocaItem, optioncount: u32, filtertags: Option<&Vec<&str>>) -> (Vec<&'a VocaItem>, u32) {
    //reserve an index for the correct option
    let correctindex: f64 = rand::random::<f64>() * (optioncount as f64);
    let correctindex: u32 = correctindex as u32;
    let mut options: Vec<&VocaItem> = Vec::new();
    for i in 0..optioncount {
        if i == correctindex {
            options.push(correctitem);
        } else {
            loop {
                let candidate  = data.pick(None, filtertags, false);
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
fn multiquiz(data: &VocaList, mut optscoredata: Option<&mut VocaScore>, choicecount: u32, phon: bool, filtertags: Option<&Vec<&str>>) {
    let instructions = "type p for phonetic transcription, x for example, q to quit, ENTER to skip";
    println!("MULTIPLE-CHOICE QUIZ ({})",instructions);
    println!("-------------------------------------------------------------------------------------------------");
    loop {
        //select a random item
        let vocaitem;
        if let Some(ref mut scoredata) = optscoredata {
            vocaitem = data.pick(Some(scoredata), filtertags, true);
        } else {
            vocaitem = data.pick(None, filtertags, true);
        }
        quizprompt(vocaitem, phon);
        let (options, correctindex) = getquizoptions(&data, &vocaitem, choicecount, filtertags);
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
                } else if response == "h" {
                    println!("{}",instructions);
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
        let first: u8 = *first - 0x31u8;
        let secondchar: char = *second as char;
        let second: u8 = *second - 0x61u8;
        if firstchar.is_ascii_digit() {
            if secondchar.is_ascii_alphabetic() {
                //println!("{}@{} in {:?}.. solved={:?}", (first as usize), (second as usize),mappings, solved); //DEBUG
                if let Some(mapped) = mappings.get(second as usize) {
                    if !solved.contains(&first) {
                        let correct: bool = *mapped == first;
                        if correct {
                            solved.push(first);
                            println!("{}", Green.paint("Correct!"));
                        } else {
                            println!("{}", Red.paint("Wrong!"));
                        }
                        if let Some(ref mut scoredata) = optscoredata {
                            if let Some(vocaitem) = vocaitems.get(first as usize) {
                                scoredata.addscore(vocaitem, correct);
                            }
                        }
                        return true;
                    } else {
                        println!("{}", Red.paint("This one was already solved!"));
                    }
                } else {
                    eprintln!("{}", Red.paint("Invalid input"));
                }
            } else {
                println!("{}", Red.paint("Expected a letter in the second position (for example: 1a)"));
            }
        } else {
            println!("{}", Red.paint("Expected a digit in the first position (for example: 1a)"));
        }
    }
    false
}

///Match quiz
fn matchquiz(data: &VocaList, mut optscoredata: Option<&mut VocaScore>, matchcount: u8, phon: bool, filtertags: Option<&Vec<&str>>) {
    println!("MATCH QUIZ (Enter a match by entering a number and a letter, enter q to quit, ENTER to skip)");
    println!("----------------------------------------------------------------------------------------");
    loop {
        let mut vocaitems: Vec<&VocaItem> = Vec::new();
        for _i in 0..matchcount {
            let vocaitem;
            if let Some(ref mut scoredata) = optscoredata {
                vocaitem = data.pick(Some(scoredata), filtertags, true);
            } else {
                vocaitem = data.pick(None, filtertags, true);
            }
            vocaitems.push(vocaitem);
        }
        //create a random order for presentation of the translations
        //values correspond to indices in vocaitems
        let mut mappings: Vec<u8> = (0..matchcount).collect();
        thread_rng().shuffle(&mut mappings);
        let mut solved: Vec<u8> = Vec::new();
        let mut solvedanswers: Vec<u8> = Vec::new();


        loop {
            for (i, vocaitem) in vocaitems.iter().enumerate() {
                if !solved.contains(&(i as u8)) {
                    if phon {
                        println!("{}) {} ({})", i+1, vocaitem.word, vocaitem.transcription);
                    } else {
                        println!("{}) {}", i+1, vocaitem.word);
                    }
                } else {
                    if let Some(solvedanswer) = mappings.iter().position(|&j| j == i as u8) {
                        solvedanswers.push(solvedanswer as u8);
                    }
                }
            }
            println!("{}", Blue.paint("---match with:---"));
            //println!("{:?}.. solved={:?}", mappings, solved); //DEBUG
            for (i, mappedindex) in mappings.iter().enumerate() {
                if !solvedanswers.contains(&(i as u8)) {
                    if let Some(vocaitem) = vocaitems.get(*mappedindex as usize) {
                        let c: char = (0x61u8 + i as u8) as char;
                        println!("{}) {}", c, vocaitem.translation);
                    }
                }
            }
            //get response from user
            if let Some(response) = getinputline() {
                if response == "q" {
                    return;
                } else {
                    if parsematchresponse(&vocaitems, &mappings, response, &mut optscoredata, &mut solved) {
                        if solved.len() == matchcount as usize {
                            break;
                        }
                    }
                }
            } else {
                break;
            }
        }
    }
}



fn main() {
    let defaultdatadir = defaultdatadir();
    let defaultscoredir = defaultscoredir();
    let arg_file = Arg::with_name("file")
                        .help("Vocabulary file to load, either a full path or from ~/.config/vocajeux/data/")
                        .index(1)
                        .required(true);
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
    let argmatches = App::new("Vocajeux")
        .version("0.1")
        .author("Maarten van Gompel (proycon) <proycon@anaproy.nl>")
        .about("Games for learning vocabulary")
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
        .arg(Arg::with_name("accesskey")
             .help("Access key")
             .long("accesskey")
             .takes_value(true)
             .short("K")
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
                    .arg(arg_file.clone())
                    .arg(arg_tags.clone())
                    .arg(arg_translations.clone())
                    .arg(arg_examples.clone())
                    .arg(arg_comments.clone())
                    .arg(Arg::with_name("showtags")
                         .help("Show tags")
                         .long("showtags")
                    )
                    .arg(arg_phon.clone()))
        .subcommand(SubCommand::with_name("csv")
                    .about("Output all data as CSV")
                    .arg(arg_file.clone())
                    .arg(arg_tags.clone()))
        .subcommand(SubCommand::with_name("add")
                    .about("Add a new word")
                    .arg(arg_file.clone())
                    .arg(Arg::with_name("word")
                        .help("The word")
                        .index(2)
                        .required(true))
                    .arg(Arg::with_name("translation")
                         .help("Translation")
                         .long("translation")
                         .takes_value(true)
                         .short("t"))
                    .arg(Arg::with_name("phon")
                         .help("Phonetic transcription")
                         .long("phon")
                         .takes_value(true)
                         .short("p"))
                    .arg(Arg::with_name("example")
                         .help("Example")
                         .long("example")
                         .takes_value(true)
                         .short("x"))
                    .arg(Arg::with_name("comment")
                         .help("Comment")
                         .long("comment")
                         .takes_value(true)
                         .short("C"))
                    .arg(Arg::with_name("tags")
                         .help("Tags (comma separated)")
                         .long("tags")
                         .takes_value(true)
                         .short("T"))
                    )
        .subcommand(SubCommand::with_name("flashcards")
                    .about("Flashcards")
                    .arg(arg_file.clone())
                    .arg(arg_tags.clone())
                    .arg(arg_phon.clone()))
        .subcommand(SubCommand::with_name("pick")
                    .about("Pick and display a random word")
                    .arg(arg_file.clone())
                    .arg(arg_tags.clone())
                    .arg(arg_phon.clone()))
                    .arg(arg_translations.clone())
                    .arg(arg_examples.clone())
                    .arg(arg_comments.clone())
        .subcommand(SubCommand::with_name("find")
                    .about("Find and display a specific word")
                    .arg(Arg::with_name("word")
                        .help("The word")
                        .index(2)
                        .required(true))
                    .arg(arg_file.clone())
                    .arg(arg_tags.clone())
                    .arg(arg_phon.clone()))
                    .arg(arg_translations.clone())
                    .arg(arg_examples.clone())
                    .arg(arg_comments.clone())
        .subcommand(SubCommand::with_name("quiz")
                    .about("Simple open quiz")
                    .arg(arg_file.clone())
                    .arg(arg_tags.clone())
                    .arg(arg_phon.clone()))
        .subcommand(SubCommand::with_name("choicequiz")
                    .about("Simple multiple-choice quiz")
                    .arg(arg_file.clone())
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
                    .arg(arg_file.clone())
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

    let datadir = PathBuf::from(argmatches.value_of("datadir").expect("Invalid data dir"));
    fs::create_dir_all(&datadir).expect("Unable to create data directory");
    let scoredir = PathBuf::from(argmatches.value_of("scoredir").expect("Invalid score dir"));
    fs::create_dir_all(&scoredir).expect("Unable to create score directory");

    if debug {
        eprintln!(" (data directory is {})", &datadir.to_str().unwrap());
        eprintln!(" (score directory is {})", &scoredir.to_str().unwrap());
    }


    match argmatches.subcommand_name() {
        None => {
            eprintln!("No command given, see --help for syntax");
            std::process::exit(1);
        },
        Some("ls") =>  {
            let dataindex = getdataindex(None);
            for file in dataindex.iter() {
                println!("{}", file.to_str().unwrap());
            }
        },
        _ => { // all other subcommands that take a file parameter
            let submatches = argmatches.subcommand_matches(argmatches.subcommand_name().unwrap()).unwrap();
            let filename = submatches.value_of("file").expect("Expected filename");
            let datafile: Option<String> = if Path::new(filename).exists() {
                eprintln!("Loading {}", filename);
                Some(filename.to_string())
            } else {
                getdatafile(filename, datadir).map(|f| f.to_str().unwrap().to_string()) //Option<PathBuf> to Option<String>, this looks a bit convoluted to me, revisit later
            };
                //This would iterate over all available files but is unnecessarily expensive
                //compared to the above:
                /*if let Some(founditem) = dataindex.iter().find(|e| e.file_stem().unwrap() == filename) {
                    datafile = founditem.to_str();
                }*/
            if datafile == None {
                eprintln!("Data file not found");
                std::process::exit(1);
            } else if debug {
                eprintln!(" (data file is {})", datafile.as_ref().unwrap());
            }
            let filebase = PathBuf::from(datafile.clone().unwrap().as_str());
            let scorefile = getscorefile(filebase.to_str().unwrap(), scoredir, submatches.value_of("accesskey"));
            if debug {
                eprintln!(" (score file is {})", scorefile.to_str().unwrap());
            }
            let filtertags: Option<Vec<&str>> = submatches.value_of("tags").map(|tagstring: &str| {
                tagstring.split_terminator(',').collect()
            });
            if let Some("add") = argmatches.subcommand_name() {
                //open writable
                let mut data = VocaList::parse(datafile.as_ref().unwrap()).expect("Unable to read data");
                let word = submatches.value_of("word").unwrap().to_string();
                let translation = submatches.value_of("translation");
                let phon = submatches.value_of("phon");
                let example = submatches.value_of("example");
                let comment = submatches.value_of("comment");
                let tags: Option<Vec<&str>> = submatches.value_of("tags").map(|tagstring: &str| {
                    tagstring.split_terminator(',').collect()
                });
                data.append(word,  translation, phon, example, comment, tags.as_ref());
                data.save(datafile.as_ref().unwrap()).expect("Unable to save");
            } else {
                //open read only
                match VocaList::parse(datafile.as_ref().unwrap()) {
                    Ok(data) => {
                        //see what subcommand to perform
                        match argmatches.subcommand_name() {
                            Some("show") => {
                                data.show(submatches.is_present("translations"), submatches.is_present("phon"), filtertags.as_ref(), submatches.is_present("showtags"), submatches.is_present("examples"), submatches.is_present("comments"));
                            },
                            Some("csv") => {
                                data.csv(filtertags.as_ref()).expect("Error during CSV serialisation");
                            },
                            Some("pick") | Some("find") | Some("quiz") | Some("choicequiz") | Some("matchquiz") | Some("flashcards") => {
                                let mut optscoredata: Option<VocaScore> = match scorefile.exists() {
                                    true => VocaScore::load(scorefile.to_str().expect("Invalid score file")).ok(),
                                    false => Some(VocaScore { ..Default::default() } ),
                                };
                                match argmatches.subcommand_name() {
                                    Some("pick") => {
                                        pick(&data, optscoredata.as_mut() , submatches.is_present("phon"), submatches.is_present("translations"), submatches.is_present("examples"), filtertags.as_ref());
                                    },
                                    Some("find") => {
                                        let word = submatches.value_of("word").expect("No word specified");
                                        find(&data, &word, optscoredata.as_mut() , submatches.is_present("phon"), submatches.is_present("translations"), submatches.is_present("examples"));
                                    },
                                    Some("choicequiz") => {
                                        if let Some(choicecount) = submatches.value_of("multiplechoice") {
                                            let choicecount: u32 = choicecount.parse().expect("Not a valid number for --multiplechoice");
                                            multiquiz(&data, optscoredata.as_mut(), choicecount, submatches.is_present("phon"), filtertags.as_ref());
                                        }
                                    },
                                    Some("matchquiz") => {
                                        if let Some(matchcount) = submatches.value_of("number") {
                                            let matchcount: u8 = matchcount.parse().expect("Not a valid number for --number");
                                            matchquiz(&data, optscoredata.as_mut(), matchcount, submatches.is_present("phon"), filtertags.as_ref());
                                        }
                                    },
                                    Some("quiz") => {
                                        quiz(&data, optscoredata.as_mut() , submatches.is_present("phon"), filtertags.as_ref());
                                    },
                                    Some("flashcards") => {
                                        flashcards(&data, optscoredata.as_mut() , submatches.is_present("phon"), filtertags.as_ref());
                                    },
                                    _ => {}
                                }
                                if let Some(ref scoredata) = optscoredata {
                                    scoredata.save(scorefile.to_str().expect("Invalid score file")).expect("Unable to save");
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
                }//match
            } //iflet
        }
    }
}
