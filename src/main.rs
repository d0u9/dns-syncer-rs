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
use tokio::signal::unix::{signal, SignalKind};
use tracing::{debug, error, info, warn};
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Options::parse();
    let mut log_level = tracing::Level::INFO;
    let mut tracing_builder = tracing_subscriber::fmt();

    tracing_builder = if let Some(level) = opts.log_level {
        log_level = level;
        tracing_builder.with_max_level(level)
    } else {
        tracing_builder.with_max_level(log_level)
    };

    if log_level >= tracing::Level::DEBUG {
        tracing_builder = tracing_builder.with_file(true).with_line_number(true);
    };

    tracing_builder.finish().init();

    info!("Thanks for using DNS Syner...");

    let (config, backends) = app_init(opts.config_file).await?;
    info!("The Configuration is: {:?}", config);

    for (i, backend) in backends.iter().enumerate() {
        debug!("Backend[{}]: {:?}", i, backend);
    }

    if config.check_interval == 0 {
        run_once(&config, &backends).await?;
    } else {
        warn!("Running in blocking mode");
        run_as_blocking(&config, &backends).await?;
    }

    Ok(())
}

async fn run_as_blocking(config: &Config, backends: &[Backend]) -> Result<()> {
    let mut interval =
        tokio::time::interval(tokio::time::Duration::from_secs(config.check_interval));

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    loop {
        let sync_result = run_once(config, backends).await;
        match sync_result {
            Ok(_) => {}
            Err(e) => error!("[Sync Failed] {:?}", e),
        }

        tokio::select! {
            _ = sigterm.recv() => { return Err(AppErr{ msg: "Receive SIGTERM, quit".to_string() } )  },
            _ = sigint.recv() => { return Err(AppErr{ msg: "Receive SIGINT, quit".to_string() } ) },
            _ = interval.tick() => {},
        };
    }
}

async fn run_once(_config: &Config, backends: &[Backend]) -> Result<()> {
    let v4addr = get_ip::get_pub_ip_v4().await?;

    for backend in backends.iter() {
        backend.sync(&v4addr).await?;
    }

    Ok(())
}
