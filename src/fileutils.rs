use std::path::Path;

use crate::logger::Logger;


const NOT_FOUND_PAGE_NAME: &str = "404.html";
const HTTP_OK_RESPONSE: &str = "HTTP/1.1 200 OK";
const HTTP_NOT_FOUND_RESPONSE: &str = "HTTP/1.1 404 NOT FOUND";


pub enum FiletypeProcessError
{
    UnsupportedFileType,
    NoExtensionFound,
}

pub fn get_filetype(filename: &String) -> Result<String, FiletypeProcessError> {
    if let Some(dot_pos) = filename.rfind('.') {
        let extension = &filename[dot_pos+1..];
        match extension {
            "html" | "htm" => Ok("text/html".to_string()),
            "jpg" | "jpeg" => Ok("image/jpeg".to_string()),
            "svg" => Ok("image/svg+xml".to_string()),
            "png" => Ok("image/png".to_string()),
            "css" => Ok("text/css".to_string()),
            "js" => Ok("application/javascript".to_string()),
            _ => Err(FiletypeProcessError::UnsupportedFileType)
        }
    } else {
        Err(FiletypeProcessError::NoExtensionFound)
    }
}


pub fn get_filename<'a>(request_method: &'a str, request_referer: Option<String>, path: &'a str) -> (&'a str, String)
{
    match request_method.find("/")
    {
        Some(value) =>
        {
            let mut file_string_points = Vec::new();

            if request_method.chars().nth(value + 1).unwrap() != ' '
            {
                for (i, char) in request_method.chars().enumerate()
                {
                    if char == ' '
                    {
                        if file_string_points.len() < 2
                        {
                            file_string_points.push(i);
                        }
                    }
                }
                
                let mut result = format!("{}{}", path, &request_method[file_string_points[0] + 2..file_string_points[1]]);

                if Path::new(&result).exists()
                {
                    (HTTP_OK_RESPONSE, result)
                }
                else
                {
                    // handle weird request
                    match request_referer
                    {
                        Some(value) =>
                        {
                            // find third and last slashes
                            let mut slash_chars = Vec::new();
                            for (i, char) in value.chars().enumerate()
                            {
                                if char == '/'
                                {
                                    slash_chars.push(i);
                                }
                            }

                            let slice;
                            match value.chars().last()
                            {
                                Some(_) => 
                                {
                                    // Get the unnecessary address part
                                    slice = &value[slash_chars[2] + 1..*slash_chars.last().unwrap() + 1];
                                },

                                None => 
                                {
                                    Logger::printmsg(Logger::ThreadErr, format!("Failed to process non-exist path"));
                                    return (HTTP_NOT_FOUND_RESPONSE, format!("{}{}", path, NOT_FOUND_PAGE_NAME));
                                }
                            }

                            // Remove the unnecessary address path to get clear path to included
                            // to html files
                            result = result.replace(slice, "");
                            return (HTTP_OK_RESPONSE, result);
                        },

                        None => (),
                    }

                    return (HTTP_NOT_FOUND_RESPONSE, format!("{}{}", path, NOT_FOUND_PAGE_NAME));
                }
            }
            else
            {
                return (HTTP_NOT_FOUND_RESPONSE, format!("{}{}", path, NOT_FOUND_PAGE_NAME));
            }
        },

        None => (HTTP_NOT_FOUND_RESPONSE, format!("{}{}", path, NOT_FOUND_PAGE_NAME))
    }
}

