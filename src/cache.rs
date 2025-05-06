use crate::lru_queue::LruQueue;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Clone)]
pub struct CacheRecord {
    pub response: String,
    pub time_now: Instant,
    pub expiry_secs: Option<u32>,
    pub date: String
}

impl CacheRecord {
    // Assume all response have Date, following specs
    pub fn new(response: String, time_now: Instant, expiry_secs: Option<u32>, date: String) -> Self {
        Self {
            response,
            time_now,
            expiry_secs,
            date
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

    pub fn get_cached(self: &mut Cache, request: &String) -> Option<(Option<CacheRecord>, bool)> {
        let mut is_expired = false;
        if !self.cache.contains_key(request) {
            return Some((None, is_expired));
        };

        let entry_ref = self.cache.get(request)?;
        // println!("Checking time out");
        // println!("Entry expiry: {:?}", entry_ref.expiry_secs);
        if self.check_time_out(&entry_ref.time_now, entry_ref.expiry_secs) {
            is_expired = true;
            return Some((Some(entry_ref.clone()), is_expired));
        }

        // If in cache, move to end of lru
        self.lru.add_lru(request);
        Some((Some(entry_ref.clone()), is_expired))
    }

    pub fn add_cache(self: &mut Cache, req: String, res: String, expiry: Option<u32>, date: String) -> bool {
        // evict if full
        let mut is_evicted = false;
        if self.cache.len() == Self::CACHE_MAX {
            if let Some(evict_elem) = self.lru.evict_lru() {
                self.cache.remove(&evict_elem);
                is_evicted = true;
            };
        }

        let time_now = Instant::now();
        self.lru.add_lru(&req);
        self.cache
            .insert(req, CacheRecord::new(res, time_now, expiry, date));

        is_evicted
    }
    pub fn remove_cache(self: &mut Cache, request: &String){
        self.cache.remove(request);
        self.lru.remove_lru(request);
    }

    // Task 3: Handle cache-control directive checking
    fn is_cache_allowed_single(self: &Cache, cache_header: &String) -> bool{
        // TODO: Is "max-age=\"0\"" valid
        !(cache_header == "private" 
            || cache_header == "no-store"
            || cache_header == "no-cache"
            || cache_header == "max-age=0"
            || cache_header == "must-validate"
            || cache_header == "proxy-revalidate")
    }

    fn is_cache_allowed_list(self: &Cache, word_list: &Vec<String>) -> bool{
        for word in word_list{
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
