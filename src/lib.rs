extern crate rand;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate regex;
extern crate md5;
extern crate dirs;
extern crate csv;

use std::fs;
use std::error::Error;
use std::fmt;
use std::io;
use std::iter::Iterator;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use md5::{compute,Digest};
use std::path::{Path,PathBuf};
use std::iter::FromIterator;

/// Vocabulary Item data structure
#[derive(Serialize, Deserialize)]
pub struct VocaItem {
    #[serde(default)] //deserialise missing fields to default empty values
    pub word: String,
    #[serde(default)]
    pub transcription: String,
    #[serde(default)]
    pub translation: String,
    #[serde(default)]
    pub example: String,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub tags: Vec<String>
}

/// Vocabulary List data structure
#[derive(Serialize, Deserialize)]
pub struct VocaList {
    pub items: Vec<VocaItem>
}

#[derive(Serialize, Deserialize)]
pub struct VocaScore {
    pub correct: HashMap<String,u32>,
    pub incorrect: HashMap<String,u32>,
    pub lastseen: HashMap<String,u64>,
//    pub due: HashMap<String,u64>
}

///we implement the Display trait so we can print VocaItems
impl fmt::Display for VocaItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.word)
    }
}

impl VocaItem {
    pub fn id(&self) -> md5::Digest {
        md5::compute(self.word.as_bytes())
    }
    pub fn id_as_string(&self) -> String {
        format!("{:x}",self.id())
    }

    pub fn filter(&self, filtertags: Option<&Vec<&str>>) -> bool {
        match filtertags {
            Some(tags) => match tags.is_empty() {
               false => {
                   //do the actual matching
                   self.tags.iter().any(|tag| tags.contains(&tag.as_str()))
               },
               true => true
            },
            None => true
        }
    }
}

impl VocaList {
    /// Parse the vocabulary data file (JSON) into the VocaList structure
    pub fn parse(filename: &str) -> Result<VocaList, Box<dyn Error>> {
        let data = fs::read_to_string(filename)?;
        let data: VocaList = serde_json::from_str(data.as_str())?; //(shadowing)
        Ok(data)
    }

    /// Add a new item to the vocabulary list
    pub fn append(&mut self, word: String, translation: Option<&str>, transcription: Option<&str>, example: Option<&str>, comment: Option<&str>, tags: Option<&Vec<&str>>) {
        let tags: Vec<String> = if let Some(ref tags) = tags {
            tags.iter()
                .map(|s| { s.to_string() })
                .collect()
        } else {
            Vec::new()
        };
        let item = VocaItem {
            word: word,
            translation: translation.map(|s:&str| s.to_string()).unwrap_or(String::new()),
            transcription: transcription.map(|s:&str| s.to_string()).unwrap_or(String::new()),
            example: example.map(|s:&str| s.to_string()).unwrap_or(String::new()),
            comment: comment.map(|s:&str| s.to_string()).unwrap_or(String::new()),
            tags: tags,
        };
        self.items.push(item);
    }

    pub fn save(&self, filename: &str) -> std::io::Result<()> {
        let data: String = serde_json::to_string(self)?;
        fs::write(filename, data)
    }

    /// Show the contents of the Vocabulary List; prints to to standard output
    pub fn show(&self, withtranslation: bool, withtranscription: bool, filtertags: Option<&Vec<&str>>, withtags: bool, withexample: bool, withcomment: bool) {
        for item in self.items.iter() {
            if item.filter(filtertags) {
                print!("{}", item);
                if withtranscription { print!("\t{}", item.transcription) }
                if withtranslation { print!("\t{}", item.translation) }
                if withexample { print!("\t{}", item.example) }
                if withcomment { print!("\t{}", item.comment) }
                if withtags {
                    print!("\t");
                    for (i, tag) in item.tags.iter().enumerate() {
                        print!("{}", tag);
                        if i < item.tags.len() - 1 {
                            print!(",")
                        }
                    }
                }
                println!()
            }
        }
    }

    ///Output all data as CSV
    pub fn csv(&self, filtertags: Option<&Vec<&str>>) -> Result<(), Box<Error>> {
        let mut wtr = csv::WriterBuilder::new()
            .flexible(true)
            .has_headers(false)
            .from_writer(io::stdout());
        for item in self.items.iter() {
            if item.filter(filtertags) {
                wtr.serialize(item)?;
            }
        };
        wtr.flush()?;
        Ok(())
    }

    ///Select a word
    pub fn pick(&self, optscoredata: Option<&VocaScore>, filtertags: Option<&Vec<&str>>) -> &VocaItem {
        let sum: f64 = self.items.iter().map(|item| {
            if item.filter(filtertags) {
                if let Some(ref scoredata) = optscoredata {
                    scoredata.score(item.id_as_string().as_str())
                } else {
                    1.0
                }
            } else {
                0.0
            }
        }).sum();
        let choice: f64 = rand::random::<f64>() * sum;
        let mut score: f64 = 0.0; //cummulative score
        let mut choiceindex: usize = 0;
        for (i, item) in self.items.iter().enumerate() {
            if item.filter(filtertags) {
                if let Some(ref scoredata) = optscoredata {
                    score += scoredata.score(item.id_as_string().as_str());
                } else {
                    score += 1.0;
                }
                if score >= choice {
                    choiceindex = i;
                    break;
                }
            }
        }
        &self.items[choiceindex]
    }
}


impl VocaScore {
    /// Load score file
    pub fn load(filename: &str) -> Result<VocaScore, Box<dyn Error>> {
        let data = fs::read_to_string(filename)?;
        let data: VocaScore = serde_json::from_str(data.as_str())?; //(shadowing)
        Ok(data)
    }

    ///Save a score file
    pub fn save(&self, filename: &str) -> std::io::Result<()> {
        let data: String = serde_json::to_string(self)?;
        fs::write(filename, data)
    }

    ///Return the 'score' for an item, this corresponds to the probability it is presented, so
    ///the lower the score, the better a word is known
    pub fn score(&self, id: &str) -> f64 {
        let correct = *self.correct.get(id).or(Some(&0)).unwrap() + 1;
        let incorrect = *self.incorrect.get(id).or(Some(&0)).unwrap() + 1;
        incorrect as f64 / correct as f64
    }

    pub fn addscore(&mut self, item: &VocaItem, correct: bool) {
        let id: String = item.id_as_string();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Unable to get time").as_secs();
        self.lastseen.insert(id.clone(),now);
        if correct {
            *self.correct.entry(id).or_insert(0) += 1;
        } else {
            *self.incorrect.entry(id).or_insert(0) += 1;
        }
    }
}

impl Default for VocaScore {
    fn default() -> VocaScore {
        VocaScore {
            correct: HashMap::new(),
            incorrect: HashMap::new(),
            lastseen: HashMap::new()
        }
    }
}


/// Return the default data directory
pub fn defaultdatadir() -> PathBuf {
    PathBuf::from(dirs::config_dir().expect("Unable to find configuration dir")).join("vocajeux").join("data")
}
///
/// Return the default score directory
pub fn defaultscoredir() -> PathBuf {
    PathBuf::from(dirs::config_dir().expect("Unable to find configuration dir")).join("vocajeux").join("scores")
}

pub fn getdatafile(name: &str, datapath: PathBuf) -> Option<PathBuf> {
    let mut filename: String = name.to_owned();
    filename.push_str(".json");
    let datafile = datapath.join(filename);
    match datafile.exists() {
        true => Some(datafile),
        false => None
    }
}

pub fn getscorefile(name: &str, scorepath: PathBuf, accesskey: Option<&str>) -> PathBuf {
    let mut filename: String = if name.ends_with(".json") {
        name[..name.len()-5].to_string()
    } else {
        name.to_string()
    };
    if let Some(accesskey) = accesskey {
        filename.push_str(".");
        filename.push_str(accesskey);
    }
    filename.push_str(".score.json");
    scorepath.join(filename)
}


/// Returns an index of available vocabulary sets
pub fn getdataindex(configpath_opt: Option<PathBuf>) -> Vec<PathBuf> {
    let mut index: Vec<PathBuf> = Vec::new();
    let configpath;
    if let Some(configpath_some) = configpath_opt {
        configpath = configpath_some;
    } else {
        configpath = dirs::config_dir().expect("Unable to find configuration dir");
    }
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
