use std::{collections::HashMap, fs::File, io::BufReader, time::{Duration, SystemTime}};

use serde::{Deserialize, Serialize};

const DATABASE_SAVE_INTERVAL_SECONDS: u64 = 60;
const VISIT_EXPIRE: u64 = Duration::from_hours(24).as_secs() as u64;
const VISIT_HISTORY_MAX_SIZE: usize = 1000;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Database {
    #[serde(default)]
    clicks: u64,
    #[serde(default)]
    review_ratings: HashMap<u8, i64>,
    #[serde(default)]
    visit_history: HashMap<u32, SystemTime>,
    #[serde(default)]
    visits: u64,
    #[serde(skip)]
    dirty_start: Option<SystemTime>,
    #[serde(skip)]
    next_expire: Option<SystemTime>,
}

impl Database {
    pub fn clicks(&self) -> u64 {
        self.clicks
    }

    fn set_dirty(&mut self) {
        if self.dirty_start.is_none() {
            self.dirty_start = Some(SystemTime::now());
        }
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

    pub fn add_visit(&mut self, ip: u32) {
        if self.visit_history.contains_key(&ip) {
            return;
        }

        if self.visit_history.len() > VISIT_HISTORY_MAX_SIZE {
            self.force_clear_oldest_visit();
        }

        self.visit_history.insert(ip, SystemTime::now());
        self.visits += 1;
        self.set_dirty();
    }

    fn force_clear_oldest_visit(&mut self) {
        let mut oldest: Option<(&u32, &SystemTime)> = None;
        for (ip, time) in &self.visit_history {
            if oldest.is_none() || *time < *oldest.unwrap().1 {
                oldest = Some((ip, time));
            }
        }
        
        if let Some((ip, _)) = oldest {
            let ip = *ip;
            self.visit_history.remove(&ip);
        }
        self.set_dirty();
    }

    fn clear_old_visits(&mut self) {
        let now = SystemTime::now();
        let mut next_expire_time = SystemTime::now().checked_add(Duration::from_hours(1000)).unwrap();
        let mut to_purge = vec![];
        for (ip, time) in self.visit_history.iter() {
            if now.duration_since(*time).unwrap().as_secs() > VISIT_EXPIRE {
                to_purge.push(*ip);
            } else {
                if next_expire_time > *time {
                    next_expire_time = *time;
                }
            }
        }
        for ip in &to_purge {
            self.visit_history.remove(ip);
        }
        self.next_expire = Some(next_expire_time);
        if to_purge.len() > 0 {
            self.set_dirty();
        }
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
            if self.db.next_expire.is_none() || self.db.next_expire.unwrap() < SystemTime::now() {
                self.db.clear_old_visits();
            }
            if dirty.elapsed().unwrap().as_secs() > DATABASE_SAVE_INTERVAL_SECONDS {
                let db_save = self.db.clone();
                std::thread::Builder::new()
                    .spawn(move || {
                        let file = std::fs::File::create("database.bin").unwrap();
                        let writer = std::io::BufWriter::new(file);
                        bincode::serialize_into(writer, &db_save).unwrap();
                        println!("Database saved");
                    })
                    .unwrap();
                self.db.dirty_start = None;
            }
        }
    }
}
