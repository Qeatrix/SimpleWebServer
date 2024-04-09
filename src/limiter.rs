use std::
{
    collections::HashMap,
    time::{Instant, Duration},
    sync::{Arc, RwLock}, thread,
};

use crate::logger::Logger;


#[derive(Debug)]
pub struct Limiter
{
    request: Arc<RwLock<HashMap<String, (Instant, u32)>>>,
    max_requests: u32,
    window: Duration,
}

impl Limiter
{
    pub fn new(max_requests: u32, window: Duration) -> Limiter
    {
        Limiter
        { 
            request: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window,
        }
    }

    pub fn check(&self, address: &str) -> bool
    {
        let requests = Arc::clone(&self.request);

        // Read phase: Lookup the entry
        let entry = 
        {
            let map_data = requests.read().expect("RwLock poisoned");
            map_data.get(address).cloned()
        };

        if let Some(entry) = entry 
        {
            // Get write lock if entry found
            let mut map_data = requests.write().expect("RwLock poisoned");

            if entry.0.elapsed() > self.window
            {
                let entry = map_data.entry(address.to_string()).or_insert((Instant::now(), 0));
                *entry = (Instant::now(), 1);
                true
            }
            else if entry.1 < self.max_requests
            {
                let entry = map_data.entry(address.to_string()).or_insert((Instant::now(), 0));
                *entry = (Instant::now(), entry.1 + 1);
                true
            }
            else
            {
                false
            }
        }
        else
        {
            let mut map_data = requests.write().expect("RwLock poisoned");

            map_data.entry(address.to_string()).or_insert((Instant::now(), 0));
            true
        }
    }

    pub fn extract_address(mut peer: String) -> Option<String>
    {
        let port_offset = match peer.find(':')
        {
            Some(result) => result,
            _ => return None,
        };

        peer.replace_range(port_offset.., "");
        Some(peer)
    }

    pub fn run_clean_cycle(&self,
                         delay: Duration,
                         max_size: usize,
                         clear_time: Duration,
                         rate_limiter: Arc<Limiter>,
                        )
    {
        thread::spawn(move ||
        {
            loop 
            {
                thread::sleep(delay);
                let mut cleaned_count = 0;
                {
                    Logger::printmsg(Logger::Info, format!("Trying to clean rate limiter hashmap..."));

                    let requests = Arc::clone(&rate_limiter.request);
                    let mut keys_to_remove = Vec::new();

                    {
                        let map_data = requests.read().expect("RwLock poisoned");
                        if map_data.len() >= max_size
                        {
                            for (key, value) in map_data.iter()
                            {
                                if value.0.elapsed() > clear_time
                                {
                                    keys_to_remove.push(key.clone());
                                }
                            }
                        }
                    }

                    let mut map_data = requests.write().expect("RwLock poisoned");
                    for key in keys_to_remove
                    {
                        map_data.remove(&key);
                        cleaned_count += 1;
                    }
                }
                Logger::printmsg(Logger::Info, format!("Limiter hashmap cleaning: cleaned {cleaned_count} entries"));
            }
        });
    }
}
