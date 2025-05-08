use crate::lru_queue::LruQueue;
use crate::request::Request;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Clone)]
pub struct CacheRecord {
    pub request: Request,
    pub response: Vec<u8>,
    pub time_now: Instant,
    pub expiry_secs: Option<u32>,
    pub date: String,
}

impl CacheRecord {
    // Assume all response have Date, following specs
    pub fn new(
        request: Request,
        response: Vec<u8>,
        time_now: Instant,
        expiry_secs: Option<u32>,
        date: String,
    ) -> Self {
        Self {
            request,
            response,
            time_now,
            expiry_secs,
            date,
        }
    }
}

pub struct Cache {
    lru: LruQueue<String>,
    cache: HashMap<String, CacheRecord>,
}

impl Cache {
    const CACHE_MAX: usize = 10;

    pub fn new() -> Self {
        Self {
            lru: LruQueue::new(),
            cache: HashMap::new(),
        }
    }

    fn check_time_out(self: &Cache, time_now: &Instant, expiry: Option<u32>) -> bool {
        // println!("Expiry: {:?}", expiry);
        let Some(expiry_secs) = expiry else {
            return false;
        };

        let elapsed_secs = time_now.elapsed().as_secs();
        // println!("Elapsed secs: {}", elapsed_secs);

        if elapsed_secs > (expiry_secs as u64) {
            return true;
        }

        false
    }

    // Returns (entry, is_expired) from the cache given the request, none if the cache doesn't exist
    pub fn get(self: &mut Cache, request: &String) -> Option<(CacheRecord, bool)> {
        let entry_ref = self.cache.get(request)?;
        if self.check_time_out(&entry_ref.time_now, entry_ref.expiry_secs) {
            return Some((entry_ref.clone(), true));
        }

        // If in cache, move to end of lru
        self.lru.add_lru(request);
        Some((entry_ref.clone(), false))
    }

    // Adds
    pub fn add_cache(
        self: &mut Cache,
        request_data: String,
        request: Request,
        response_data: Vec<u8>,
        expiry: Option<u32>,
        date: String,
    ) -> Option<CacheRecord> {
        let mut evicted = None;

        // evict if full
        if self.cache.len() == Self::CACHE_MAX {
            // remove lru
            let evicted_key = self.lru.evict_lru().unwrap();
            evicted = Some(self.cache.get(&evicted_key).unwrap().clone());
            self.cache.remove(&evicted_key);
        }

        let time_now = Instant::now();
        self.lru.add_lru(&request_data);
        self.cache.insert(
            request_data,
            CacheRecord::new(request, response_data, time_now, expiry, date),
        );

        evicted
    }

    pub fn remove_cache(self: &mut Cache, request: &String) -> CacheRecord {
        self.lru.evict_lru_by_value(request);
        let record = self.cache.get(request).unwrap().clone();
        self.cache.remove(request);
        record
    }

    // Task 3: Handle cache-control directive checking
    fn is_cache_allowed_single(self: &Cache, cache_header: &String) -> bool {
        // TODO: Is "max-age=\"0\"" valid
        !(cache_header == "private"
            || cache_header == "no-store"
            || cache_header == "no-cache"
            || cache_header == "max-age=0"
            || cache_header == "must-validate"
            || cache_header == "proxy-revalidate")
    }

    fn is_cache_allowed_list(self: &Cache, word_list: &Vec<String>) -> bool {
        for word in word_list {
            if !self.is_cache_allowed_single(word) {
                return false;
            }
        }

        true
    }

    pub fn is_cache_allowed(self: &Cache, word_list: &Vec<String>) -> bool {
        // println!("Split cache control: {:?}", word_list);
        return self.is_cache_allowed_list(word_list);
    }
}
