use crate::err::*;
use serde_json::Value as JsonVal;

pub struct Restful;

enum ReqMethod {
    Get,
    Post,
    Patch,
}

impl Restful {
    fn request_builder(
        url: &str,
        method: ReqMethod,
        headers: Option<Vec<(&str, &str)>>,
    ) -> Result<reqwest::RequestBuilder> {
        let client = reqwest::Client::new();

        let mut builder = match method {
            ReqMethod::Get => client.get(url),
            ReqMethod::Post => client.post(url),
            ReqMethod::Patch => client.patch(url),
        };

        builder = builder.header("Content-Type", "application/json");

        if let Some(headers) = headers {
            for header in headers.into_iter() {
                builder = builder.header(header.0, header.1);
            }
        }

        Ok(builder)
    }

    pub async fn get(url: &str, headers: Option<Vec<(&str, &str)>>) -> Result<serde_json::Value> {
        let builder = Self::request_builder(url, ReqMethod::Get, headers)?;

        let body = builder.send().await?.text().await?;

        let json: serde_json::Value = serde_json::from_str(&body)?;

        Ok(json)
    }

    pub async fn post(
        url: &str,
        headers: Option<Vec<(&str, &str)>>,
        json: &JsonVal,
    ) -> Result<serde_json::Value> {
        let builder = Self::request_builder(url, ReqMethod::Post, headers)?;

        let body = builder.json(json).send().await?.text().await?;

        let json: serde_json::Value = serde_json::from_str(&body)?;

        Ok(json)
    }

    pub async fn patch(
        url: &str,
        headers: Option<Vec<(&str, &str)>>,
        json: &JsonVal,
    ) -> Result<serde_json::Value> {
        let builder = Self::request_builder(url, ReqMethod::Patch, headers)?;

        let resp = builder.json(json).send().await?;

        let body = resp.text().await?;

        let json: serde_json::Value = serde_json::from_str(&body)?;

        Ok(json)
    }
}
