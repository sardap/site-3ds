use std::{collections::HashMap, fs::File, io::BufReader, time::SystemTime};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Database {
    #[serde(default)]
    clicks: u64,
    #[serde(default)]
    review_ratings: HashMap<u8, i64>,
    #[serde(default)]
    visits: u64,
    #[serde(skip)]
    dirty_start: Option<SystemTime>,
}

impl Database {
    pub fn clicks(&self) -> u64 {
        self.clicks
    }

    fn set_dirty(&mut self) {
        self.dirty_start = Some(SystemTime::now());
    }

    pub fn set_clicks(&mut self, clicks: u64) {
        self.clicks = clicks;
        self.set_dirty();
    }

    pub fn get_review_ratings(&self) -> HashMap<u8, i64> {
        self.review_ratings.clone()
    }

    pub fn get_review_rating(&self, id: u8) -> i64 {
        *self.review_ratings.get(&id).unwrap_or(&0)
    }

    pub fn add_review_rating(&mut self, id: u8, rating: i64) {
        match self.review_ratings.get_mut(&id) {
            Some(entry) => {
                *entry += rating;
            }
            None => {
                self.review_ratings.insert(id, rating);
            }
        }
        self.set_dirty();
    }

    pub fn get_visits(&self) -> u64 {
        self.visits
    }

    pub fn increment_visits(&mut self) {
        self.visits += 1;
        self.set_dirty();
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
        if let Some(dirty) = self.db.dirty_start {
            if dirty.elapsed().unwrap().as_secs() > 5 {
                let file = std::fs::File::create("database.bin").unwrap();
                let writer = std::io::BufWriter::new(file);
                bincode::serialize_into(writer, &self.db).unwrap();
                println!("Database saved");
                self.db.dirty_start = None;
            }
        }
    }
}
