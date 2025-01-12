use clap::Parser;
use regex::Regex;

#[derive(Parser, Debug)]
pub struct Args {
  #[arg(long)]
  pub client_id: String,
  #[arg(long)]
  pub access_token: String,
  #[arg(short, long)]
  pub category_name: String,
  #[arg(short, long, default_value = "600")]
  pub batch_size: usize,
  #[arg(short, long)]
  pub filter: Regex,
}
