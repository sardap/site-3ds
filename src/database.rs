use std::{collections::HashMap, fs::File, io::BufReader};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Database {
    #[serde(default)]
    clicks: u64,
    #[serde(default)]
    review_ratings: HashMap<u8, i128>,
    #[serde(skip)]
    pub dirty: bool,
}

impl Database {
    pub fn clicks(&self) -> u64 {
        self.clicks
    }

    pub fn set_clicks(&mut self, clicks: u64) {
        self.clicks = clicks;
        self.dirty = true;
    }

    pub fn get_review_ratings(&self) -> HashMap<u8, i128> {
        self.review_ratings.clone()
    }

    pub fn get_review_rating(&self, id: u8) -> i128 {
        *self.review_ratings.get(&id).unwrap_or(&0)
    }

    pub fn add_review_rating(&mut self, id: u8, rating: i128) {
        match self.review_ratings.get_mut(&id) {
            Some(entry) => {
                *entry += rating;
            }
            None => {
                self.review_ratings.insert(id, rating);
            }
        }
        self.dirty = true;
    }
}

pub struct DatabaseHolder {
    pub db: Database,
}

impl DatabaseHolder {
    pub fn new() -> DatabaseHolder {
        if let Ok(file) = std::fs::File::open("database.bin") {
            let reader = std::io::BufReader::new(file);
            if let Ok(db) = bincode::deserialize_from::<BufReader<File>, Database>(reader) {
                return DatabaseHolder { db };
            };
        }

        DatabaseHolder {
            db: Database::default(),
        }
    }

    pub fn step(&mut self) {
        if self.db.dirty {
            let file = std::fs::File::create("database.bin").unwrap();
            let writer = std::io::BufWriter::new(file);
            bincode::serialize_into(writer, &self.db).unwrap();
            self.db.dirty = false;
        }
    }
}
