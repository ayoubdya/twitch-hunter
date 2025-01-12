use clap::Parser;
use std::{error::Error, sync::Arc};
use tokio::sync::mpsc::channel;

mod helix;
use helix::TwitchHelix;

mod irc;
use irc::TwitchIrc;

mod args;
use args::Args;

// lazy_static! {
//   pub static ref REGEX_FILTER: Regex = Regex::new(r"https://.+").unwrap();
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let args = Args::parse();
  println!("{:?}", args);
  // std::process::exit(0);

  // let client_id = std::env::var("TWITCH_CLIENT_ID").expect("TWITCH_CLIENT_ID not set");
  // let access_token = std::env::var("TWITCH_ACCESS_TOKEN").expect("TWITCH_ACCESS_TOKEN not set");
  let helix = TwitchHelix::new(args.client_id, args.access_token);

  let category_id = match helix.get_category_id(args.category_name.as_str()).await {
    Ok(Some(id)) => id,
    Ok(None) => {
      eprintln!("Category not found");
      std::process::exit(1);
    }
    Err(e) => {
      eprintln!("Error: {}", e);
      std::process::exit(1);
    }
  };

  // Marvel Rivals 1264310518
  // let category_id = "1264310518".to_owned();

  let streams = helix.get_streams(category_id.as_str()).await?;
  println!("Found {} streams", streams.len());

  let (tx, mut rx) = channel(100);

  let regex_filter = Arc::new(args.filter);

  let batch_size = args.batch_size;
  for i in 0..streams.len() / batch_size + 1 {
    println!("Spawning batch {}", i);

    let tx = tx.clone();
    let regex_filter = regex_filter.clone();

    let streams_batch = streams
      .iter()
      .skip(i * batch_size)
      .take(batch_size)
      .map(|s| s.user_login.clone())
      .collect();

    tokio::spawn(async move {
      let mut irc = TwitchIrc::new(tx, streams_batch, regex_filter).await;
      irc.run().await.unwrap();
    });
  }

  while let Some(msg) = rx.recv().await {
    println!("{} | {}: {}", msg.channel, msg.nickname, msg.msg);
  }

  Ok(())
}
