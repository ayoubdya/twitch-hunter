use clap::Parser;
use regex::Regex;

#[derive(Parser)]
pub struct Args {
  #[arg(long)]
  pub client_id: String,
  #[arg(long)]
  pub access_token: String,
  #[arg(short, long)]
  pub category_name: String,
  #[arg(short, long, default_value = "100")]
  pub batch_size: usize,
  #[arg(short, long, value_name = "REGEX")]
  pub filter: Regex,
}
