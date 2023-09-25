use std::collections::HashMap;

use super::restful::*;
use crate::backends::DNSSync;
use crate::err::*;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;

#[derive(Debug, Serialize, Deserialize)]
pub struct Cloudflare {
    #[serde(rename(serialize = "authentication", deserialize = "authentication"))]
    auth: Auth,
    zones: Vec<Zone>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Zone {
    id: String,
    records: Vec<Record>,
}

#[derive(Debug)]
enum Action {
    Patch(String, JsonValue),
    Post(JsonValue),
}

impl Action {
    async fn do_action(&self, zone_id: &str, auth: &Auth) -> Result<()> {
        match self {
            Self::Post(data) => Self::post(zone_id, auth, data).await?,
            Self::Patch(id, data) => Self::patch(zone_id, auth, id, data).await?,
        }

        Ok(())
    }

    async fn patch(zone_id: &str, auth: &Auth, record_id: &str, data: &JsonValue) -> Result<()> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            zone_id, record_id,
        );

        let (k, v) = auth.http_header();
        let headers: Vec<(&str, &str)> = vec![(&k, &v)];

        Restful::patch(url.as_str(), Some(headers), data).await?;

        Ok(())
    }

    async fn post(zone_id: &str, auth: &Auth, data: &JsonValue) -> Result<()> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone_id,
        );

        let (k, v) = auth.http_header();
        let headers: Vec<(&str, &str)> = vec![(&k, &v)];

        Restful::post(url.as_str(), Some(headers), data).await?;

        Ok(())
    }
}

impl Zone {
    async fn list_recordds(&self, auth: &Auth) -> Result<Vec<Record>> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            self.id,
        );

        let (k, v) = auth.http_header();
        let headers: Vec<(&str, &str)> = vec![(&k, &v)];
        let resp = Restful::get(&url, Some(headers)).await?;

        let mut h: HashMap<String, serde_json::Value> = serde_json::from_value(resp)?;
        let result = h.remove("result").ok_or(AppErr {
            msg: String::from(
                "list dns records, and there is no 'result' filed in the returned json",
            ),
        })?;

        let dns_records: Vec<Record> = serde_json::from_value(result)?;
        Ok(dns_records)
    }

    async fn get_actions_by_diff(
        &self,
        v4addr: &str,
        remote_records: Vec<Record>,
    ) -> Result<Vec<Action>> {
        let mut actions: Vec<Action> = Vec::new();

        for local in self.records.iter() {
            let mut processed = false;

            for remote in remote_records.iter() {
                let mut patch: Record = Default::default();
                let mut need_update = false;

                if local.name != remote.name {
                    continue;
                }

                if local.dns_type != remote.dns_type {
                    println!(
                        "dns type changed from {:?} to {:?}",
                        remote.dns_type, local.dns_type
                    );
                    need_update = true;
                }

                if (local.content.is_empty() && v4addr != remote.content)
                    || (!local.content.is_empty() && local.content != remote.content)
                {
                    println!(
                        "content change from {} to {}",
                        remote.content, local.content
                    );
                    need_update = true;
                }

                // ttl == 1 means auto
                match (&local.ttl, &remote.ttl) {
                    (Some(local), Some(remote)) if local != remote => {
                        println!("ttl changed from {:?} to {:?}", remote, local);
                        need_update = true;
                        patch.ttl = Some(local.to_owned());
                    }
                    (_, None) => {
                        println!("[BUG] remote ttl is NONE");
                    }
                    _ => {}
                };

                match (&local.proxied, &remote.proxied) {
                    (Some(local), Some(remote)) if local != remote => {
                        println!("proxied changed from {:?} to {:?}", remote, local);
                        need_update = true;
                        patch.proxied = Some(local.to_owned());
                    }
                    (_, None) => {
                        println!("[BUG] remote proxied is NONE");
                    }
                    _ => {}
                }

                match (&local.comment, &remote.comment) {
                    (Some(local), Some(remote)) if local != remote => {
                        println!("comment changed from {:?} to {:?}", remote, local);
                        need_update = true;
                        patch.comment = Some(local.clone());
                    }
                    _ => {}
                }

                if need_update {
                    // name is a required field
                    patch.name = local.name.clone();
                    // type is a required field
                    patch.dns_type = local.dns_type.to_owned();
                    patch.content = if local.content.is_empty() {
                        // content is a required field
                        v4addr.to_owned()
                    } else {
                        // content is a required field
                        local.content.clone()
                    };

                    let json = serde_json::to_value(patch)?;
                    actions.push(Action::Patch(remote.id.clone(), json));
                }

                processed = true
            }

            if processed {
                continue;
            }

            let mut local_clone = local.clone();
            if local_clone.content.is_empty() {
                local_clone.content = v4addr.to_owned();
            }
            if local_clone.ttl.is_none() {
                local_clone.ttl = Some(1);
            }
            if local_clone.proxied.is_none() {
                local_clone.proxied = Some(false);
            }

            let json = serde_json::to_value(local_clone)?;
            actions.push(Action::Post(json));
        }

        Ok(actions)
    }

    async fn do_actions(&self, auth: &Auth, actions: Vec<Action>) -> Result<()> {
        for action in actions.into_iter() {
            println!("===> Action ==> {:?}", action);
            action.do_action(&self.id, auth).await?;
        }
        Ok(())
    }

    async fn sync(&self, auth: &Auth, v4addr: &str) -> Result<()> {
        let remote_records = self.list_recordds(auth).await?;
        let actions = self.get_actions_by_diff(v4addr, remote_records).await?;
        self.do_actions(auth, actions).await?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct Record {
    #[serde(skip_serializing, default)]
    id: String,

    #[serde(rename(serialize = "type", deserialize = "type"))]
    dns_type: DNSType,

    name: String,

    #[serde(default)]
    content: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    proxied: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    ttl: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
enum DNSType {
    Uninit,
    A,
    AAAA,
    CNAME,
}

impl Default for DNSType {
    fn default() -> Self {
        Self::Uninit
    }
}

impl ToString for DNSType {
    fn to_string(&self) -> String {
        match self {
            Self::Uninit => "uninit",
            Self::A => "A",
            Self::AAAA => "AAAA",
            Self::CNAME => "CNAME",
        }
        .to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Auth {
    ApiToken {
        api_token: String,
    },
    ApiKey {
        api_key: String,
        account_email: String,
    },
}

impl Auth {
    fn http_header(&self) -> (String, String) {
        match self {
            Self::ApiToken { api_token } => {
                let k = "Authorization".to_string();
                let v = format!("  Bearer {}", api_token);
                (k, v)
            }
            Self::ApiKey {
                api_key: _,
                account_email: _,
            } => {
                todo!()
            }
        }
    }
}

impl Cloudflare {
    pub fn from_yaml_value(yaml: YamlValue) -> Result<Self> {
        let rval: Self = serde_yaml::from_value(yaml)?;
        Ok(rval)
    }
}

#[async_trait]
impl DNSSync for Cloudflare {
    async fn sync(&self, cur_v4addr: &str) -> Result<()> {
        for zone in self.zones.iter() {
            zone.sync(&self.auth, cur_v4addr).await?;
        }
        Ok(())
    }
}
