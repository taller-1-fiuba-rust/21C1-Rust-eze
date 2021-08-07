use std::{collections::HashMap, io::Write};

use crate::server_html::error::http_error::HttpError;
use crate::server_html::status_codes::status_code::StatusCode;

#[derive(Debug, PartialEq, Clone)]
pub struct HttpResponse {
    version: String,
    status_code: String,
    status_text: String,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
}
impl Default for HttpResponse {
    fn default() -> Self {
        Self {
            version: "HTTP/1.1".into(),
            status_code: "200".into(),
            status_text: "OK".into(),
            headers: None,
            body: None,
        }
    }
}
impl HttpResponse {
    pub fn new(
        status_code: StatusCode,
        headers: Option<HashMap<String, String>>,
        body: Option<String>,
    ) -> HttpResponse {
        let mut response: HttpResponse = HttpResponse::default();

        let (code, text) = status_code.take_info();
        response.status_code = code;
        response.status_text = text;

        response.headers = match &headers {
            Some(_h) => headers,
            None => {
                let mut h = HashMap::new();
                h.insert("Content-Type".to_string(), "text/html".to_string());
                Some(h)
            }
        };

        response.body = body;
        response
    }
    pub fn send_response(&self, write_stream: &mut impl Write) -> Result<(), HttpError> {
        let res = self.clone();
        let response_string: String = String::from(res);
        let _ = write!(write_stream, "{}", response_string);
        Ok(())
    }
}
impl HttpResponse {
    fn version(&self) -> String {
        self.version.clone()
    }
    fn status_code(&self) -> String {
        self.status_code.clone()
    }
    fn status_text(&self) -> String {
        self.status_text.clone()
    }
    fn headers(&self) -> String {
        let map: HashMap<String, String> = self.headers.clone().unwrap();
        let mut header_string: String = "".into();
        for (k, v) in map.iter() {
            header_string = format!("{}{}:{}\r\n", header_string, k, v);
        }
        header_string
    }
    pub fn body(&self) -> &str {
        match &self.body {
            Some(b) => b.as_str(),
            None => "",
        }
    }
}

impl From<HttpError> for HttpResponse {
    fn from(err: HttpError) -> HttpResponse {
        let (code, description) = err.take();
        Self {
            version: "HTTP/1.1".into(),
            status_code: code,
            status_text: description,
            headers: None,
            body: None,
        }
    }
}

impl From<HttpResponse> for String {
    fn from(res: HttpResponse) -> String {
        let res1 = res.clone();
        format!(
            "{} {} {}\r\n{}Content-Length: {}\r\n\r\n{}",
            &res1.version(),
            &res1.status_code(),
            &res1.status_text(),
            &res1.headers(),
            &res.body.unwrap_or_default().len(),
            &res1.body()
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::server_html::status_codes::status_code;
    #[test]
    fn test_response_struct_creation_200() {
        let response_actual = HttpResponse::new(
            status_code::defaults::ok(),
            None,
            Some("Item was shipped on 21st Dec 2020".into()),
        );
        let response_expected = HttpResponse {
            version: "HTTP/1.1".to_string(),
            status_code: "200".to_string(),
            status_text: "OK".to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("Content-Type".to_string(), "text/html".to_string());
                Some(h)
            },
            body: Some("Item was shipped on 21st Dec 2020".into()),
        };
        assert_eq!(response_actual, response_expected);
    }
    #[test]
    fn test_response_struct_creation_404() {
        let response_actual = HttpResponse::new(
            status_code::defaults::not_found(),
            None,
            Some("Item was shipped on 21st Dec 2020".into()),
        );
        let response_expected = HttpResponse {
            version: "HTTP/1.1".to_string(),
            status_code: "404".to_string(),
            status_text: "Not Found".to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("Content-Type".to_string(), "text/html".to_string());
                Some(h)
            },
            body: Some("Item was shipped on 21st Dec 2020".into()),
        };
        assert_eq!(response_actual, response_expected);
    }
    #[test]
    fn test_http_response_creation() {
        let response_expected = HttpResponse {
            version: "HTTP/1.1".to_string(),
            status_code: "404".to_string(),
            status_text: "Not Found".to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("Content-Type".to_string(), "text/html".to_string());
                Some(h)
            },
            body: Some("Item was shipped on 21st Dec 2020".into()),
        };
        let http_string: String = response_expected.into();
        let response_actual = "HTTP/1.1 404 Not Found\r\nContent-Type:text/html\r\nContent-Length: 33\r\n\r\nItem was shipped on 21st Dec 2020";
        assert_eq!(http_string, response_actual);
    }
}
