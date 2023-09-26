use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    #[arg(short, long)]
    pub config_file: String,
    #[arg(long)]
    pub log_level: Option<tracing::Level>,
}
