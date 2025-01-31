use std::{collections::HashMap, fs::File, io::BufReader, net::IpAddr, time::SystemTime, u32};

use serde::{Deserialize, Serialize};

const DATABASE_SAVE_INTERVAL_SECONDS: u64 = 60;
const VISIT_HISTORY_MAX_SIZE: usize = 5000;
const DATABASE_FILENAME: &str = "site_3ds_database.bin";

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

#[derive(Serialize, Deserialize, Clone)]
pub struct Database {
    review_ratings: HashMap<u8, i64>,
    visit_history: HashMap<StoredIp, u32>,
    least_visitor: Option<LeastVisitor>,
    visits: u64,
    #[serde(skip)]
    dirty_start: Option<SystemTime>,
}

impl Default for Database {
    fn default() -> Self {
        Database {
            review_ratings: HashMap::with_capacity(u8::MAX as usize),
            visit_history: HashMap::with_capacity(VISIT_HISTORY_MAX_SIZE),
            least_visitor: None,
            visits: 0,
            dirty_start: None,
        }
    }
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
                if self.visit_history.len() + 1 > VISIT_HISTORY_MAX_SIZE
                    && self.least_visitor.is_some()
                {
                    self.visit_history.remove(&self.least_visitor.unwrap().ip);
                    self.least_visitor = None;
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
                let mut iter = self.visit_history.iter();
                for i in 0..10 {
                    if let Some((ip, count)) = iter.next() {
                        if i == 0 {
                            self.least_visitor = Some(LeastVisitor {
                                ip: *ip,
                                count: *count,
                            });
                        } else {
                            if *count < self.least_visitor.unwrap().count {
                                self.least_visitor = Some(LeastVisitor {
                                    ip: *ip,
                                    count: *count,
                                });
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        self.set_dirty();
    }

    pub fn new() -> Database {
        if let Ok(file) = std::fs::File::open(DATABASE_FILENAME) {
            let reader = std::io::BufReader::new(file);
            if let Ok(db) = bincode::deserialize_from::<BufReader<File>, Database>(reader) {
                println!("Loading existing database");
                return db;
            };
        }

        println!("Creating new database");
        Database::default()
    }

    pub fn step(&mut self) {
        if let Some(dirty) = self.dirty_start {
            if dirty.elapsed().unwrap().as_secs() > DATABASE_SAVE_INTERVAL_SECONDS {
                {
                    let file = std::fs::File::create(DATABASE_FILENAME).unwrap();
                    let writer = std::io::BufWriter::new(file);
                    bincode::serialize_into(writer, &self).unwrap();
                }
                println!("Database saved");
                self.dirty_start = None;
            }
        }
    }
}
