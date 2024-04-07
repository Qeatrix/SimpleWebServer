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

        match self
        {
            Logger::Request => println!("{:?} [{}] > {}", timestamp, Self::build_color("REQ", Colors::Green as u32), msg),
            Logger::Thread => println!("{:?} [{}] > {}", timestamp, Self::build_color("THR", Colors::Yellow as u32), msg),
            Logger::Worker => println!("{:?} [{}] > {}", timestamp, Self::build_color("WOR", Colors::Blue as u32), msg),
            Logger::Info => println!("{:?} [{}] > {}", timestamp, Self::build_color("INF", Colors::White as u32), msg),
            Logger::RequestErr => println!("{:?} [{}][{}] > {}", timestamp, Self::build_color("REQ", Colors::Red as u32), Self::build_color("ERR", Colors::Red as u32), msg),
            Logger::ThreadErr => println!("{:?} [{}][{}] > {}", timestamp, Self::build_color("THR", Colors::Red as u32), Self::build_color("ERR", Colors::Red as u32), msg),
            Logger::WorkerErr => println!("{:?} [{}][{}] > {}", timestamp, Self::build_color("WOR", Colors::Red as u32), Self::build_color("ERR", Colors::Red as u32), msg),
            Logger::InfoErr => println!("{:?} [{}][{}] > {}", timestamp, Self::build_color("INF", Colors::Red as u32), Self::build_color("ERR", Colors::Red as u32), msg),
        }
    }

    fn build_color(msg: &str, number: u32) -> String
    {
        let mut chars = String::from("\x1b[");
        let closing_chars = "m";
        chars.push_str(&number.to_string());
        chars.push_str(&closing_chars);

       chars.push_str(msg);
       chars.push_str("\x1b[0m");
       chars
    }
}
