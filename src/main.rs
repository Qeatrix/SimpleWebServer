use std::
{
  net::{TcpListener, TcpStream},
  io::{BufReader, BufRead, Write, ErrorKind}, fs,
  env, time::Duration, thread,
  sync::Arc,
};

mod limiter;
use limiter::Limiter;

mod logger;
use logger::Logger;

use webserver::ThreadPool;

mod config;


const BIND_ADDRESS: &str = "127.0.0.1:7878";

// Limiter settings
const MAX_REQUESTS: u32 = 100;
const MAX_REQUESTS_WINDOW_DURATION: Duration = Duration::from_secs(3600);
const LIMITER_CLEAN_DELAY: Duration = Duration::from_secs(3600);
const LIMITER_CLEAN_ELAPSED: Duration = Duration::from_secs(3600);
const LIMITER_CLEAN_MAXSIZE: usize = 150;


fn main()
{
    let args: Vec<String> = env::args().collect();
    let config = config::Config::build(&args);

    let listener = TcpListener::bind(BIND_ADDRESS).expect(&format!("Cannot start the server on {}", BIND_ADDRESS));
    Logger::printmsg(Logger::Info, format!("Server is started on {}", BIND_ADDRESS));

    let rate_limiter = Arc::new(Limiter::new(MAX_REQUESTS, MAX_REQUESTS_WINDOW_DURATION));

    let rate_limiter_clone = rate_limiter.clone();
    thread::spawn(move || rate_limiter_clone.clean_hashmap(LIMITER_CLEAN_DELAY, LIMITER_CLEAN_MAXSIZE, LIMITER_CLEAN_ELAPSED));

    let pool = ThreadPool::new(20);

    for stream in listener.incoming()
    {
        let stream = stream.unwrap();

        let mut stream_peer = stream.peer_addr().unwrap().to_string();
        stream_peer = match Limiter::extract_address(stream_peer)
        {
            Some(result) => result,
            _ => {
                Logger::printmsg(Logger::InfoErr, String::from("Couldn't extract the ip address."));
                continue;
            },
        };

        let stream_peer_limit = rate_limiter.check(&stream_peer);
        if stream_peer_limit == false
        {
            Logger::printmsg(Logger::Info, String::from(format!("Request has been locked from {}", stream_peer)));
            continue;
        };

        let path = config.file_path.clone();

        pool.execute(|| 
        {
            match handle_connection(stream, path)
            {
                Ok(()) => (),
                Err(msg) => panic!("{}", msg),
            }
        });
    }
}

fn handle_connection(mut stream: TcpStream, path: String) -> Result<(), String>
{
    let buf_reader = BufReader::new(&stream);
    let full_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    if full_request.len() == 0
    {
        Logger::printmsg(Logger::RequestErr, String::from("Got zero length request"));
        return Ok(());
    }

    let request_method = &full_request[0];


    //println!("Method: {}\nResonse: {:#?}", request_method, full_request);
    
    let (status_line, filename) = if request_method == "GET / HTTP/1.1"
    {
        Logger::printmsg(Logger::Request, format!("Connection established to {}, responsed with \"200 OK\"", &stream.peer_addr().unwrap()));
        ("HTTP/1.1 200 OK", path + "index.html")
    }
    else
    {
        Logger::printmsg(Logger::Request, format!("Connection established to {}, responsed with \"404 NOT FOUND\"", &stream.peer_addr().unwrap()));
        ("HTTP/1.1 404 NOT FOUND", path + "404.html")
    };

    let content = match fs::read_to_string(filename.clone())
    {
        Ok(file) => file,
        Err(error) => match error.kind()
        {
            ErrorKind::NotFound => {
                return Err(String::from(format!("File \"{}\" not found in this directory.", filename)))
            }
            _ => return Err(String::from(format!("Cannot open the file \"{}\".", filename))),
        }
    };

    let lenght = content.len();

    let response = format!("{status_line}\r\nContent-Length: {lenght}\r\n\r\n{content}");

    stream.write_all(response.as_bytes()).expect("Stream was interrupted.");
    Ok(())
}

