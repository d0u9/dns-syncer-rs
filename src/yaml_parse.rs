use crate::backends::{Backend, Cloudflare};
use crate::err::*;

use serde::Deserialize;
use serde_yaml::Value as YamlValue;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct ConfigBackend {
    provider: String,
    #[serde(flatten)]
    object: YamlValue,
}

#[derive(Debug, Deserialize)]
struct ConfigYaml {
    check_interval: u32,
    backends: Vec<ConfigBackend>,
}

pub struct Config {
    pub check_interval: u32,
}

impl ConfigYaml {
    pub fn from_yaml<P>(file: P) -> Result<ConfigYaml>
    where
        P: AsRef<Path>,
    {
        let file = File::open(file).expect("Open config file failed");
        let file_reader = BufReader::new(file);

        let rval = serde_yaml::from_reader(file_reader)?;
        Ok(rval)
    }

    pub fn new_config(&self) -> Config {
        Config {
            check_interval: self.check_interval,
        }
    }

    pub fn create_backends(self) -> Result<Vec<Backend>> {
        let mut backends = Vec::new();

        for backend_yaml in self.backends.into_iter() {
            let backend = match backend_yaml.provider.to_lowercase().as_str() {
                "cloudflare" => {
                    let cloudlare = Cloudflare::from_yaml_value(backend_yaml.object)?;
                    Backend::Cloudflare(cloudlare)
                }
                _ => {
                    return Err(AppErr {
                        msg: format!("unknown backend {}", backend_yaml.provider),
                    })
                }
            };

            backends.push(backend);
        }

        Ok(backends)
    }
}

pub async fn app_init<P>(config_file: P) -> Result<(Config, Vec<Backend>)>
where
    P: AsRef<Path>,
{
    let conf_yaml = ConfigYaml::from_yaml(config_file)?;

    let config = conf_yaml.new_config();
    let backends = conf_yaml.create_backends()?;

    Ok((config, backends))
}
