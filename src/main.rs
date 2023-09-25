mod backends;
mod err;
mod get_ip;
mod options;
mod yaml_parse;

use crate::backends::{Backend, DNSSync};
use crate::err::*;
use crate::options::Options;
use crate::yaml_parse::*;

use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Options::parse();
    let (config, backends) = app_init(opts.config_file).await?;

    if config.check_interval == 0 {
        run_once(config, backends).await?;
    }

    Ok(())
}

async fn run_as_deamon() -> Result<()> {
    Ok(())
}

async fn run_once(config: Config, backends: Vec<Backend>) -> Result<()> {
    let v4addr = get_ip::get_pub_ip_v4().await?;

    for backend in backends.iter() {
        backend.sync(&v4addr).await.unwrap();
    }

    Ok(())
}
