use std::error::Error;
use tokio::sync::mpsc::{channel, Sender};

mod helix;
use helix::TwitchHelix;

mod irc;
use irc::TwitchIrc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let client_id = std::env::var("TWITCH_CLIENT_ID").expect("TWITCH_CLIENT_ID not set");
  let access_token = std::env::var("TWITCH_ACCESS_TOKEN").expect("TWITCH_ACCESS_TOKEN not set");
  let helix = TwitchHelix::new(client_id, access_token);

  let category_id = helix
    .get_category_id("Rust")
    .await
    .unwrap()
    .expect("Could not find category");

  // let category_id = "263490".to_owned();

  let streams = helix.get_streams(category_id.as_str()).await?;
  println!("Found {} streams", streams.len());

  // let channels: Vec<String> = streams.into_iter().map(|s| s.user_login).collect();
  // let mut irc = TwitchIrc::new(channels).await;
  // irc.run().await?;

  let (tx, mut rx) = channel(100);

  for channel in streams.into_iter() {
    let tx = tx.clone();
    tokio::spawn(async move {
      let mut irc = TwitchIrc::new(tx, channel.user_login).await;
      irc.run().await.unwrap();
    });
  }

  while let Some(msg) = rx.recv().await {
    println!("{}", msg);
  }

  Ok(())
}
