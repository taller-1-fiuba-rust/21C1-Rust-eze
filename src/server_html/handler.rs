use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};

use super::error::http_error::HttpError;
use super::http_response::HttpResponse;
use super::request::http_method::HttpMethod;
use crate::server_html::html_content::get_page_content;
use crate::server_html::request::{http_request::HttpRequest, http_url::HttpUrl};
use crate::server_html::status_codes::status_code;

pub trait Handler {
    fn handle(req: &HttpRequest) -> Result<HttpResponse, HttpError>;

    fn load_file(file_name: &str) -> Result<Option<Vec<u8>>, HttpError> {
        if file_name.is_empty() {
            return Ok(None);
        }

        let file_name = format!("src/server_html/resource/{}", file_name);
        let mut buff_image = Vec::new();
        let mut file = File::open(&file_name)
            .map_err(|_| HttpError::from(status_code::defaults::not_found()))?;
        file.read_to_end(&mut buff_image)
            .map_err(|_| HttpError::from(status_code::defaults::not_found()))?;
        file.flush()
            .map_err(|_| HttpError::from(status_code::defaults::not_found()))?;
        Ok(Some(buff_image))
    }
}

pub struct CommandRedisPage;

// TODO: para lo de MARTO seguramente acá no deberiamos respstar el trait HAndelr... habrá que pasar channels de alguna manera jeee
impl Handler for CommandRedisPage {
    fn handle(req: &HttpRequest) -> Result<HttpResponse, HttpError> {
        let default_command = "";
        let command = req
            .get_body()
            .unwrap_or(&default_command.to_string())
            .split('=')
            .collect::<Vec<&str>>()
            .get(1)
            .unwrap_or(&default_command)
            .to_string()
            .replace("+", " ");

        // TODO: ACA VA LO DE MARTO

        let contents = get_page_content(&command).into_bytes();

        Ok(HttpResponse::new(
            status_code::defaults::ok(),
            None,
            Some(contents),
        ))
    }
}

pub struct StaticPage;

impl Handler for StaticPage {
    fn handle(req: &HttpRequest) -> Result<HttpResponse, HttpError> {
        if req.get_method() != &HttpMethod::Get {
            return Err(HttpError::from(status_code::defaults::bad_request()));
        }

        let HttpUrl::Path(s) = req.get_url();

        let route: Vec<&str> = s.split('/').collect();
        match route[1] {
            "" => Ok(HttpResponse::new(
                status_code::defaults::ok(),
                None,
                Self::load_file("index.html")?,
            )),
            path => {
                let mut map: HashMap<String, String> = HashMap::new();
                if path.ends_with(".css") {
                    map.insert("Content-Type".to_string(), "text/css".to_string());
                } else if path.ends_with(".png") {
                    map.insert("Content-Type".to_string(), "image/png".to_string());
                } else if path.ends_with(".html") {
                    map.insert("Content-Type".to_string(), "text/html".to_string());
                } else {
                    return Ok(HttpResponse::new(
                        status_code::defaults::not_found(),
                        None,
                        StaticPage::load_file("404.html")?,
                    ));
                }

                Ok(HttpResponse::new(
                    status_code::defaults::ok(),
                    Some(map),
                    Self::load_file(path)?,
                ))
            }
        }
    }
}