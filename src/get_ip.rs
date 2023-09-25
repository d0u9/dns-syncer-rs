use std::io::{BufRead, BufReader};

use crate::err::*;

pub(crate) async fn get_pub_ip_v4() -> Result<String> {
    let resp = reqwest::get("https://1.1.1.1/cdn-cgi/trace").await?;

    let status = resp.status();
    let body = resp.text().await?;

    if status != 200 {
        return Err(AppErr {
            msg: format!("Request ip address failed: {}", body),
        });
    }

    let reader = BufReader::new(body.as_bytes());

    let ip = reader.lines().map_while(|line| line.ok()).find_map(|line| {
        if let Some((k, v)) = line.split_once('=') {
            if k == "ip" {
                Some(v.to_owned())
            } else {
                None
            }
        } else {
            None
        }
    });

    ip.ok_or(AppErr {
        msg: "cannot get public ip".to_string(),
    })
}
