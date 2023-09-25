pub type Result<T> = std::result::Result<T, AppErr>;

#[derive(Debug)]
pub struct AppErr {
    pub msg: String,
}

impl From<reqwest::Error> for AppErr {
    fn from(value: reqwest::Error) -> Self {
        Self {
            msg: value.to_string(),
        }
    }
}

impl From<std::net::AddrParseError> for AppErr {
    fn from(value: std::net::AddrParseError) -> Self {
        Self {
            msg: value.to_string(),
        }
    }
}

impl From<serde_json::Error> for AppErr {
    fn from(value: serde_json::Error) -> Self {
        Self {
            msg: format!("[serde json]: {:?}", value),
        }
    }
}

impl From<serde_yaml::Error> for AppErr {
    fn from(value: serde_yaml::Error) -> Self {
        Self {
            msg: format!("[serde yaml]: {:?}", value),
        }
    }
}
