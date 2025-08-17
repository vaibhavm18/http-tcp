use regex::Regex;
use std::{collections::HashMap, fmt, str::FromStr};

#[derive(Debug, PartialEq)]
pub enum Method {
    Get,
    Post,
    Delete,
    Put,
}

impl FromStr for Method {
    type Err = HttpRequestError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            "DELETE" => Ok(Method::Delete),
            "PUT" => Ok(Method::Put),
            _ => Err(HttpRequestError::InvalidMethod(s.to_string())),
        }
    }
}

#[derive(Debug)]
pub enum HttpRequestError {
    MissingMethod,
    MissingPath,
    MissingVersion,
    InvalidMethod(String),
    InvalidPath(String),
    InvalidVersion(String),
}

impl fmt::Display for HttpRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpRequestError::MissingMethod => write!(f, "Missing HTTP method"),
            HttpRequestError::MissingPath => write!(f, "Missing HTTP path"),
            HttpRequestError::MissingVersion => write!(f, "Missing HTTP version"),
            HttpRequestError::InvalidMethod(m) => write!(f, "Unsupported HTTP method: {}", m),
            HttpRequestError::InvalidPath(m) => write!(f, "Unsupported HTTP path: {}", m),
            HttpRequestError::InvalidVersion(m) => write!(f, "Unsupported HTTP version: {}", m),
        }
    }
}

impl std::error::Error for HttpRequestError {}

pub struct HttpRequestBuilder {
    method: Option<String>,
    path: Option<String>,
    version: Option<String>,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
}

impl HttpRequestBuilder {
    pub fn method(mut self, method: String) -> Self {
        self.method = Some(method);
        self
    }

    pub fn path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }

    pub fn version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    pub fn body(mut self, body: Option<Vec<u8>>) -> Self {
        self.body = body;
        self
    }

    pub fn build(self) -> Result<HttpRequest, HttpRequestError> {
        // Method
        let method_str = self.method.ok_or(HttpRequestError::MissingMethod)?;
        let method = method_str.parse::<Method>()?;

        // Path
        let path = self.path.ok_or(HttpRequestError::MissingPath)?;
        if !is_valid_path(&path) {
            return Err(HttpRequestError::InvalidPath(path));
        }

        // Version
        let version = self.version.ok_or(HttpRequestError::MissingVersion)?;
        if !version.starts_with("HTTP/") {
            return Err(HttpRequestError::InvalidVersion(version));
        }

        Ok(HttpRequest {
            method,
            path,
            version,
            headers: self.headers,
            body: self.body,
        })
    }
}

fn is_valid_path(path: &str) -> bool {
    if let Ok(re) = Regex::new("^/(?:[A-Za-z0-9_-]+(?:/[A-Za-z0-9_-]+)*)?$") {
        return re.is_match(path);
    }

    false
}

#[derive(Debug, PartialEq)]
pub struct HttpRequest {
    method: Method,
    path: String,
    version: String,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>, // Changed to Vec<u8> to handle binary data
}

impl HttpRequest {
    pub fn builder() -> HttpRequestBuilder {
        HttpRequestBuilder {
            method: None,
            path: None,
            version: None,
            headers: HashMap::new(),
            body: None,
        }
    }
}
