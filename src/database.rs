use std::{collections::HashMap, fs::File, io::BufReader, net::IpAddr, time::SystemTime, u32};

use serde::{Deserialize, Serialize};

const DATABASE_SAVE_INTERVAL_SECONDS: u64 = 60;
const VISIT_HISTORY_MAX_SIZE: usize = 5000;

#[derive(Serialize, Deserialize, Clone, Copy, Hash, Eq, PartialEq)]
pub enum StoredIp {
    V4(u32),
    V6(u128),
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct LeastVisitor {
    ip: StoredIp,
    count: u32,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Database {
    #[serde(default)]
    review_ratings: HashMap<u8, i64>,
    #[serde(default)]
    visit_history: HashMap<StoredIp, u32>,
    #[serde(default)]
    least_visitor: Option<LeastVisitor>,
    #[serde(default)]
    visits: u64,
    #[serde(skip)]
    dirty_start: Option<SystemTime>,
}

impl Database {
    fn set_dirty(&mut self) {
        if self.dirty_start.is_none() {
            self.dirty_start = Some(SystemTime::now());
        }
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

    pub fn add_visit(&mut self, ip: &IpAddr) {
        let ip = match ip {
            // Im not sure if it makes sense to hash the ipv4 address It probably is better to not mix hashed and not hashed data.
            IpAddr::V4(ipv4_addr) => StoredIp::V4(ipv4_addr.clone().into()),
            IpAddr::V6(ipv6_addr) => StoredIp::V6(ipv6_addr.clone().into()),
        };

        let user_visits = match self.visit_history.get_mut(&ip) {
            Some(entry) => {
                *entry = (*entry).checked_add(1).unwrap_or(u32::MAX);
                *entry
            }
            None => {
                if self.visit_history.len() > VISIT_HISTORY_MAX_SIZE {
                    self.clear_least_common_visit();
                }
                self.visit_history.insert(ip, 1);
                self.visits += 1;
                1
            }
        };

        match &mut self.least_visitor {
            Some(least_visitor) => {
                if user_visits < least_visitor.count {
                    least_visitor.ip = ip;
                    least_visitor.count = user_visits;
                }
            }
            None => {
                self.least_visitor = Some(LeastVisitor {
                    ip,
                    count: user_visits,
                });
            }
        }

        self.set_dirty();
    }

    fn clear_least_common_visit(&mut self) {
        let mut min = u32::MAX;
        let mut min_ip = None;
        for (ip, count) in &self.visit_history {
            if *count < min || min_ip.is_none() {
                min = *count;
                min_ip = Some(ip);
            }
        }
        if let Some(ip) = min_ip {
            let ip = *ip;
            self.visit_history.remove(&ip);
        }
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
                println!("Loading existing database");
                return DatabaseHolder { db };
            };
        }

        println!("Creating new database");
        DatabaseHolder {
            db: Database::default(),
        }
    }

    pub fn step(&mut self) {
        if let Some(dirty) = self.db.dirty_start {
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
