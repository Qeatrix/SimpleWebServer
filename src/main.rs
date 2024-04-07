use std::{
  net::{TcpListener, TcpStream},
  io::{BufReader, BufRead, Write}, fs,
};

use webserver::ThreadPool;

const BIND_ADDRESS: &str = "127.0.0.1:7878";
const DEFAULT_RESPONSE_HTML: &str = "response.html";

fn main()
{
    let listener = TcpListener::bind(BIND_ADDRESS).unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming()
    {
        let stream = stream.unwrap();

        pool.execute(||
        {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream)
{
    let buf_reader = BufReader::new(&stream);
    let full_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let http_head = &full_request[0];

    println!("Connection established to {}", &stream.peer_addr().unwrap());
    println!("\nMethod: {}\nResonse: {:#?}", http_head, full_request);

    let (status_line, filename) = if http_head == "GET / HTTP/1.1"
    {
        ("HTTP/1.1 200 OK", DEFAULT_RESPONSE_HTML)
    }
    else
    {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let content = fs::read_to_string(filename).unwrap();
    let lenght = content.len();

    let response = format!("{status_line}\r\nContent-Length: {lenght}\r\n\r\n{content}");

    stream.write_all(response.as_bytes()).unwrap();
}

