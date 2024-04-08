use chrono::Local;

#[allow(dead_code)]
pub enum Logger
{
    Request,
    Thread,
    Worker,
    Info,
    RequestErr,
    ThreadErr,
    WorkerErr,
    InfoErr,
}

enum Colors
{
    Red = 91,
    Green = 92,
    Yellow= 93,
    Blue = 96,
    White = 97,
}

impl Logger
{
    pub fn printmsg(self, msg: String)
    {
        let timestamp = Local::now();
        let ow = 14; //msg type offset width

        match self
        {
            Logger::Request => println!("{:?} {:>ow$} > {}", timestamp, Self::build_msgtype("REQ", Colors::Green as u32), msg),
            Logger::Thread => println!("{:?} {:>ow$} > {}", timestamp, Self::build_msgtype("THR", Colors::Yellow as u32), msg),
            Logger::Worker => println!("{:?} {:>ow$} > {}", timestamp, Self::build_msgtype("WOR", Colors::Blue as u32), msg),
            Logger::Info => println!("{:?} {:>ow$} > {}", timestamp, Self::build_msgtype("INF", Colors::White as u32), msg),
            Logger::RequestErr => println!("{:?} {:>ow$}{:>ow$} > {}", timestamp, Self::build_msgtype("REQ", Colors::Red as u32), Self::build_msgtype("ERR", Colors::Red as u32), msg),
            Logger::ThreadErr => println!("{:?} {:>ow$}{:>ow$} > {}", timestamp, Self::build_msgtype("THR", Colors::Red as u32), Self::build_msgtype("ERR", Colors::Red as u32), msg),
            Logger::WorkerErr => println!("{:?} {:>ow$}{:>ow$} > {}", timestamp, Self::build_msgtype("WOR", Colors::Red as u32), Self::build_msgtype("ERR", Colors::Red as u32), msg),
            Logger::InfoErr => println!("{:?} {:>ow$}{:>ow$} > {}", timestamp, Self::build_msgtype("INF", Colors::Red as u32), Self::build_msgtype("ERR", Colors::Red as u32), msg),
        }
    }

    fn build_msgtype(msg: &str, number: u32) -> String
    {
        let start_type_char = '[';
        let end_type_char = ']';

        let mut chars = String::from("\x1b[");
        let closing_chars = "m";
        chars.push_str(&number.to_string());
        chars.push_str(&closing_chars);

        chars.push_str(msg);
        chars.push_str("\x1b[0m");

        let mut result = start_type_char.to_string();
        result.push_str(&chars);
        result.push_str(&end_type_char.to_string());

        result
    }
}