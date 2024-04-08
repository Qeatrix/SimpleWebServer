use std::
{
    collections::HashMap,
    time::{Instant, Duration},
    sync::Mutex,
};


#[derive(Debug)]
pub struct Limiter
{
    request: Mutex<HashMap<String, (Instant, u32)>>,
    max_requests: u32,
    window: Duration,
}

impl Limiter
{
    pub fn new(max_requests: u32, window: Duration) -> Limiter
    {
        Limiter
        { 
            request: Mutex::new(HashMap::new()),
            max_requests,
            window,
        }
    }

    pub fn check(&self, address: &str) -> bool
    {
        let mut requests = self.request.lock().unwrap();
        let (last_instant, count) = requests.get(address).copied().unwrap_or((Instant::now(), 0));

        if last_instant.elapsed() > self.window
        {
            requests.insert(address.to_string(), (Instant::now(), 1));
            true
        }
        else if count < self.max_requests
        {
            requests.insert(address.to_string(), (last_instant, count + 1));
            true
        }
        else
        {
            false
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
}
