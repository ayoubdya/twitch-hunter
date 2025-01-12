use lazy_static::lazy_static;
use regex::Regex;
use std::error::Error;
use tokio::sync::mpsc::channel;

mod helix;
use helix::TwitchHelix;

mod irc;
use irc::TwitchIrc;

lazy_static! {
  pub static ref REGEX_FILTER: Regex = Regex::new(r"https://.+").unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let client_id = std::env::var("TWITCH_CLIENT_ID").expect("TWITCH_CLIENT_ID not set");
  let access_token = std::env::var("TWITCH_ACCESS_TOKEN").expect("TWITCH_ACCESS_TOKEN not set");
  let helix = TwitchHelix::new(client_id, access_token);

  lazy_static! {
    pub static ref REGEX_FILTER: Regex = Regex::new(r"https://.+").unwrap();
  }

  // let category_id = helix
  //   .get_category_id("Rust")
  //   .await
  //   .unwrap()
  //   .expect("Could not find category");

  // Marvel Rivals 1264310518
  let category_id = "1264310518".to_owned();

  let streams = helix.get_streams(category_id.as_str()).await?;
  println!("Found {} streams", streams.len());

  let (tx, mut rx) = channel(100);

  let batch_size = 600;
  for i in 0..streams.len() / batch_size + 1 {
    println!("Spawning batch {}", i);
    let tx = tx.clone();

    let streams_batch = streams
      .iter()
      .skip(i * batch_size)
      .take(batch_size)
      .map(|s| s.user_login.clone())
      .collect();

    tokio::spawn(async move {
      let mut irc = TwitchIrc::new(tx, streams_batch).await;
      irc.run().await.unwrap();
    });
  }

  while let Some(msg) = rx.recv().await {
    println!("{}", msg);
  }

  Ok(())
}
