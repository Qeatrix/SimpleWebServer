use std::{
  net::{TcpListener, TcpStream},
  io::{BufReader, BufRead, Write, ErrorKind}, fs,
};

use webserver::ThreadPool;

mod verbose;
type VerboseItem = verbose::Logger;


const BIND_ADDRESS: &str = "127.0.0.1:7878";
const DEFAULT_RESPONSE_HTML: &str = "response.html";


fn main()
{
    let listener = TcpListener::bind(BIND_ADDRESS).expect(&format!("Cannot start the server on {}", BIND_ADDRESS));
    let pool = ThreadPool::new(4);

    for stream in listener.incoming()
    {
        let stream = stream.unwrap();

        pool.execute(||
        {
            match handle_connection(stream)
            {
                Ok(()) => (),
                Err(msg) => panic!("{}", msg),
            }
        });
    }
}

fn handle_connection(mut stream: TcpStream) -> Result<(), String>
{
    let buf_reader = BufReader::new(&stream);
    let full_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    if full_request.len() == 0
    {
        VerboseItem::printmsg(VerboseItem::RequestErr, String::from("Got zero length request"));
        return Ok(());
    }

    let request_method = &full_request[0];


    VerboseItem::printmsg(VerboseItem::Request, format!("Connection established to {}", &stream.peer_addr().unwrap()));
    //println!("Method: {}\nResonse: {:#?}", request_method, full_request);

    let (status_line, filename) = if request_method == "GET / HTTP/1.1"
    {
        ("HTTP/1.1 200 OK", DEFAULT_RESPONSE_HTML)
    }
    else
    {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let content = match fs::read_to_string(filename)
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

