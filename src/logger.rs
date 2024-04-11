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
        let mut timestamp = Local::now().to_string();
        timestamp = match timestamp.find(".")
        {
            Some(result) => 
            {
                timestamp.replace_range(result.., "");
                match timestamp.find(" ")
                {
                    Some(result) =>
                    {
                        let mut start_str = timestamp[0..result].to_string();
                        let end_str = timestamp[result + 1..].to_string();
                        start_str.push('T');
                        start_str.push_str(&end_str);
                        start_str
                    },
                    None =>
                    {
                        Logger::build_msgtype("FAILED TO PROCESS TIME STAMP", Colors::Red)
                    }
                }
            },

            None => {
                Logger::build_msgtype("FAILED TO PROCESS TIME STAMP", Colors::Red)
            },
        };


        match self
        {
            Logger::Request => println!("{} {} > {}", timestamp, Self::build_msgtype("REQ", Colors::Green), msg),
            Logger::Thread => println!("{} {} > {}", timestamp, Self::build_msgtype("THR", Colors::Yellow), msg),
            Logger::Worker => println!("{} {} > {}", timestamp, Self::build_msgtype("WOR", Colors::Blue), msg),
            Logger::Info => println!("{} {} > {}", timestamp, Self::build_msgtype("INF", Colors::White), msg),
            Logger::RequestErr => println!("{} {}{} > {}", timestamp, Self::build_msgtype("REQ", Colors::Red), Self::build_msgtype("ERR", Colors::Red), msg),
            Logger::ThreadErr => println!("{} {}{} > {}", timestamp, Self::build_msgtype("THR", Colors::Red), Self::build_msgtype("ERR", Colors::Red), msg),
            Logger::WorkerErr => println!("{} {}{} > {}", timestamp, Self::build_msgtype("WOR", Colors::Red), Self::build_msgtype("ERR", Colors::Red), msg),
            Logger::InfoErr => println!("{} {}{} > {}", timestamp, Self::build_msgtype("INF", Colors::Red), Self::build_msgtype("ERR", Colors::Red), msg),
        }
    }

    fn build_msgtype(msg: &str, number: Colors) -> String
    {
        let start_type_char = '[';
        let end_type_char = ']';

        let mut chars = String::from("\x1b[");
        let closing_chars = "m";
        
        chars.push_str(&(number as u32).to_string());
        chars.push_str(&closing_chars);

        chars.push_str(msg);
        chars.push_str("\x1b[0m");

        let mut result = start_type_char.to_string();
        result.push_str(&chars);
        result.push_str(&end_type_char.to_string());

        result
    }
}
