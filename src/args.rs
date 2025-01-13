use clap::{ArgGroup, Parser};
use regex::Regex;

#[derive(Parser)]
#[command(group(
    ArgGroup::new("source")
        .required(true)
        .args(&["category_name", "streams"]),
))]
pub struct Args {
  #[arg(long)]
  pub client_id: Option<String>,
  #[arg(long)]
  pub access_token: Option<String>,
  #[arg(short, long, group = "source")]
  pub category_name: Option<String>,
  #[arg(
    short,
    long,
    group = "source",
    value_delimiter = ',',
    value_name = "STREAM1, STREAM2 ..."
  )]
  pub streams: Option<Vec<String>>,
  #[arg(short, long, default_value = "100")]
  pub batch_size: usize,
  #[arg(short, long, value_name = "REGEX")]
  pub filter: Regex,
  #[arg(long)]
  pub save: bool,
}
