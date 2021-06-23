use chrono::{DateTime, NaiveDateTime, Utc};
use rocksdb::{DB, IteratorMode};
use serde_json;
use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::{Arc, RwLock};

mod meta;
pub use meta::User;

pub mod config;

pub struct UserCtlRaw {
    pub meta: meta::MetaCtl,
    db: DB,
}
pub type UserCtl = Arc<RwLock<UserCtlRaw>>;

fn open_database(db_path: &std::path::PathBuf) -> rocksdb::DB {
    let mut opts = rocksdb::Options::default();
    opts.create_missing_column_families(true);
    opts.create_if_missing(true);
    opts.increase_parallelism(4);
    opts.set_compression_type(rocksdb::DBCompressionType::Snappy);
    opts.optimize_for_point_lookup(1024);
    opts.set_bytes_per_sync(1024 * 1024);


    rocksdb::DB::open(&opts, db_path).unwrap()
}

fn read_be_i64(input: &[u8]) -> i64 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<i64>());
    i64::from_be_bytes(int_bytes.try_into().unwrap())
}

impl UserCtlRaw {
    pub fn new(cfg: &config::Config) -> UserCtlRaw {
        let ctl = UserCtlRaw {
            meta: meta::init(&cfg.meta_path),
            db: open_database(&cfg.db_path),
        };
        ctl
    }

    pub fn read_latest(&mut self, start: usize, num: usize) -> Result<Vec<HashMap<String, String>>, String> {
        let mut ret = Vec::new();

        let iter = self.db.iterator(IteratorMode::End);
        for (idx, (key, value)) in iter.enumerate() {
            let key = read_be_i64(&key);
            let dt = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(key, 0), Utc);

            if idx >= start {
                let value_str = match std::str::from_utf8(&value) {
                    Err(msg) => return Err(msg.to_string()),
                    Ok(s) => s,
                };

                match serde_json::from_str(value_str) {
                    Err(msg) => return Err(msg.to_string()),
                    Ok(s) => {
                        let s: HashMap<String, String> = s;
                        println!("key: {:?}, dt: {:?}, timestamp: {:?}", key, dt, s["timestamp"]);
                        ret.push(s)
                    },
                };
            }

            if idx == start + num {
                break
            }
        }

        Ok(ret)
    }
}

pub fn init(cfg: &config::Config) -> UserCtl {
    Arc::new(RwLock::new(UserCtlRaw::new(cfg)))
}
