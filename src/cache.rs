use crate::lru_queue::LruQueue;
use std::collections::HashMap;
use std::time::Instant;

pub struct CacheRecord {
    response: String,
    time_now: Instant,
    expiry_secs: Option<u32>,
}

impl CacheRecord {
    pub fn new(response: String, time_now: Instant, expiry_secs: Option<u32>) -> Self {
        Self {
            response,
            time_now,
            expiry_secs,
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
        let Some(expiry_secs) = expiry else {
            return false;
        };

        let elapsed_secs = time_now.elapsed().as_secs();

        if elapsed_secs > (expiry_secs as u64) {
            return false;
        }

        true
    }

    pub fn get_cached(self: &mut Cache, request: &String) -> Option<(Option<String>, bool)> {
        let mut is_expired = false;
        if !self.cache.contains_key(request) {
            return Some((None, is_expired));
        };

        let entry_ref = self.cache.get(request)?;

        if self.check_time_out(&entry_ref.time_now, entry_ref.expiry_secs) {
            is_expired = true;
            // Remove from cache (and so the lru)
            // TODO: Check if we are allowed here
            self.cache.remove(request);
            self.lru.remove_lru(request);
            return Some((None, is_expired));
        }

        // If in cache, move to end of lru
        self.lru.add_lru(request);
        Some((Some(entry_ref.response.clone()), is_expired))
    }

    pub fn add_cache(self: &mut Cache, req: String, res: String, expiry: Option<u32>) -> bool {
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
            .insert(req, CacheRecord::new(res, time_now, expiry));

        is_evicted
    }
}
