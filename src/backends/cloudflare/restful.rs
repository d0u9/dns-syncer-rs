use crate::err::*;
use reqwest::Response;
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

    async fn non_200_to_error(resp: Response) -> Result<String> {
        let status = resp.status();
        if status != 200 {
            return Err(AppErr {
                msg: format!("Not a 200 response: {}", resp.text().await?),
            });
        }
        let body = resp.text().await?;
        Ok(body)
    }

    pub async fn get(url: &str, headers: Option<Vec<(&str, &str)>>) -> Result<serde_json::Value> {
        let builder = Self::request_builder(url, ReqMethod::Get, headers)?;

        let resp = builder.send().await?;
        let body = Self::non_200_to_error(resp).await?;
        let json: serde_json::Value = serde_json::from_str(&body)?;

        Ok(json)
    }

    pub async fn post(
        url: &str,
        headers: Option<Vec<(&str, &str)>>,
        json: &JsonVal,
    ) -> Result<serde_json::Value> {
        let builder = Self::request_builder(url, ReqMethod::Post, headers)?;

        let resp = builder.json(json).send().await?;
        let body = Self::non_200_to_error(resp).await?;
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
        let body = Self::non_200_to_error(resp).await?;
        let json: serde_json::Value = serde_json::from_str(&body)?;

        Ok(json)
    }
}
