extern crate rand;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate regex;
extern crate md5;
extern crate dirs;

use std::fs;
use std::error::Error;
use std::fmt;
use std::iter::Iterator;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use md5::{compute,Digest};
use std::path::{Path,PathBuf};

/// Vocabulary Item data structure
#[derive(Serialize, Deserialize)]
pub struct VocaItem {
    pub word: String,
    pub transcription: String,
    pub translation: String,
    pub example: String,
    pub comment: String,
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

//we implement the Display trait so we can print VocaItems
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
}

impl VocaList {
    /// Parse the vocabulary data file (JSON) into the VocaList structure
    pub fn parse(filename: &str) -> Result<VocaList, Box<dyn Error>> {
        let data = fs::read_to_string(filename)?;
        let data: VocaList = serde_json::from_str(data.as_str())?; //(shadowing)
        Ok(data)
    }

    /// List/Print the contents of the Vocabulary List to standard output
    pub fn list(&self, withtranslation: bool, withtranscription: bool) {
        for item in self.items.iter() {
            print!("{}", item);
            if withtranscription { print!("\t{}", item.transcription) }
            if withtranslation { print!("\t{}", item.translation) }
            println!()
        }
    }

    ///Select a word
    pub fn pick(&self, optscoredata: Option<&VocaScore>) -> &VocaItem {
        if let Some(ref scoredata) = optscoredata {
            let sum: f64 = self.items.iter().map(|item| {
                scoredata.score(item.id_as_string().as_str())
            }).sum();
            let choice: f64 = rand::random::<f64>() * sum;
            let mut score: f64 = 0.0; //cummulative score
            let mut choiceindex: usize = 0;
            for (i, item) in self.items.iter().enumerate() {
                score += scoredata.score(item.id_as_string().as_str());
                if score >= choice {
                    choiceindex = i;
                    break;
                }
            }
            &self.items[choiceindex]
        } else {
            let choice: f64 = rand::random::<f64>() * (self.items.len() as f64);
            let choice: usize = choice as usize;
            &self.items[choice]
        }
    }
}


impl VocaScore {
    /// Load score file
    pub fn load(filename: &str) -> Result<VocaScore, Box<dyn Error>> {
        let data = fs::read_to_string(filename)?;
        let data: VocaScore = serde_json::from_str(data.as_str())?; //(shadowing)
        Ok(data)
    }

    pub fn save(&self, filename: &str) -> std::io::Result<()> {
        let data: String = serde_json::to_string(self)?;
        fs::write(filename, data)
    }

    //Return the 'score' for an item, this corresponds to the probability it is presented, so
    //the lower the score, the better a word is known
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

/// Returns an index of available vocabulary sets
pub fn getdataindex(configpath_opt: Option<PathBuf>) -> Vec<PathBuf> {
    let mut index: Vec<PathBuf> = Vec::new();
    let configpath;
    if let Some(configpath_some) = configpath_opt {
        configpath = configpath_some;
    } else {
        configpath = dirs::config_dir().unwrap();
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
