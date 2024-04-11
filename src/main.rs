use std::
{
  net::{TcpListener, TcpStream},
  io::{BufReader, BufRead, Write},
  fs,
  env,
  time::Duration, 
  sync::Arc,
};

pub mod limiter;
use crate::limiter::Limiter;

pub mod logger;
use logger::*;

pub mod fileutils;
use fileutils::{get_filetype, get_filename};

use webserver::ThreadPool;

pub mod config;


const BIND_ADDRESS: &str = "0.0.0.0:7878";

const DEFAULT_RESPONSE_PAGE_NAME: &str = "index.html";

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
    rate_limiter.run_clean_cycle(LIMITER_CLEAN_DELAY, LIMITER_CLEAN_MAXSIZE, LIMITER_CLEAN_ELAPSED, Arc::clone(&rate_limiter));

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
            Logger::printmsg(Logger::Info, format!("Request has been blocked from {}", stream.peer_addr().unwrap().to_string()));
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

fn handle_connection(stream: TcpStream, path: String) -> Result<(), String>
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

    let mut request_referer: Option<String> = None;
    for value in full_request.iter()
    {
        if value.contains("Referer")
        {
            request_referer = Some(value.to_string());
            break;
        }
    }

    let (status_line, filename) = if request_method == "GET / HTTP/1.1"
    {
        ("HTTP/1.1 200 OK", path + DEFAULT_RESPONSE_PAGE_NAME)
    }
    else
    {
        get_filename(request_method, request_referer, &path)
    };

    match get_filetype(&filename)
    {
        Ok(req_type) =>
        {
            if req_type.contains("text") || req_type.contains("javascript")
            {
                text_to_stream(filename, status_line, &stream);
            }
            else if req_type.contains("image")
            {
                image_to_stream(filename, req_type, status_line, &stream);
            }
        },

        Err(e) =>
        {
            match e
            {
                fileutils::FiletypeProcessError::NoExtensionFound => {
                    Logger::printmsg(Logger::RequestErr, "No request acceptable extension found, trying to send data as text".to_string());
                    text_to_stream(filename, status_line, &stream); 
                },

                fileutils::FiletypeProcessError::UnsupportedFileType => {
                    Logger::printmsg(Logger::RequestErr, "Requested acceptable file type is unsupported, trying to send data as text".to_string());
                    text_to_stream(filename, status_line, &stream); 
                }
            }
        }
    }

    match status_line.find("200 OK")
    {
        Some(_) => Logger::printmsg(Logger::Request, format!("Connection established to {}, responsed with \"200 OK\"", &stream.peer_addr().unwrap())),
        None => Logger::printmsg(Logger::Request, format!("Connection established to {}, responsed with \"404 NOT FOUND\"", &stream.peer_addr().unwrap())),
    }

    Ok(())
}

fn text_to_stream(filename: String, status_line: &str, mut stream: &TcpStream)
{
    let content = fs::read_to_string(&filename).unwrap();
    let length = content.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{content}");
    stream.write_all(response.as_bytes()).expect("Stream was interrupted.");
}

fn image_to_stream(filename: String, req_type: String, status_line: &str, mut stream: &TcpStream)
{
    let content = fs::read(&filename).unwrap();
    let length = content.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\nContent-Type: {req_type}\r\n\r\n");
    stream.write(response.as_bytes()).unwrap();
    stream.write(&content).unwrap();
}
