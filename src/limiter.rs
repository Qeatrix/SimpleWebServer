use std::
{
    collections::HashMap,
    time::{Instant, Duration},
    sync::{Arc, Mutex, RwLock}, thread,
};

use crate::logger::Logger;


#[derive(Debug)]
pub struct Limiter
{
    request: Arc<RwLock<HashMap<String, Mutex<(Instant, u32)>>>>,
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

        let map_data = requests.read().expect("RwLock poisoned");
        if let Some(value) = map_data.get(address)
        {
            let mut value = value.lock().expect("Mutex poisoned");

            if value.0.elapsed() > self.window
            {
                *value = (Instant::now(), 1);
                return true
            }
            else if value.1 < self.max_requests
            {
                *value = (Instant::now(), value.1 + 1);
                return true
            }
            else
            {
                return false
            }

        }
        else
        {
            drop(map_data);

            // Create new entry in hashmap
            requests.write().expect("RwLock poisoned").insert(address.to_string(), Mutex::new((Instant::now(), 0)));
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

    pub fn clean_hashmap(&self, delay: Duration, max_size: usize, clear_time: Duration)
    {
        loop
        {
            thread::sleep(delay);
            let mut cleaned_count = 0;

            {
                Logger::printmsg(Logger::Info, format!("Trying to clean rate limiter hashmap..."));

                let requests = Arc::clone(&self.request);
                let mut keys_to_remove = Vec::new();

                {
                    let map_data = requests.read().expect("RwLock poisoned");
                    if map_data.len() >= max_size
                    {
                        for (key, value) in map_data.iter()
                        {
                            let value = value.lock().expect("Mutex poisoned");
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


                println!("{:#?}", map_data);
            }
            Logger::printmsg(Logger::Info, format!("Limiter hashmap cleaning: cleaned {cleaned_count} entries"));
        }
    }
}
