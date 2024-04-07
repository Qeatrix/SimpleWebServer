use std::env;


pub struct Config
{
    pub file_path: String,
}

impl Config
{
    pub fn build(args: &[String]) -> Config
    {
        let mut path = String::from(env::current_dir().unwrap().into_os_string().into_string().unwrap());
        path.push_str("/");

        if args.len() < 2
        {
            println!("Using default targer dir: {}", path);
            return Config { file_path: path };
        }

        let mut file_path = args[1].clone();

        if file_path.chars().last().expect("Cannot read argument") != '/'
        {
           file_path.push_str("/");
        }

        if file_path.chars().next().unwrap() != '/'
        {
            path.push_str(&file_path);
        }
        else
        {
            path = file_path;
        }

        Config { file_path: path }
    }
}
