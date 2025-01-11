use std::error::Error;

use futures::StreamExt;
use irc::client::prelude::*;
use tokio;

pub struct TwitchIrc {
  pub client: Client,
}

impl TwitchIrc {
  pub async fn new(channels: Vec<String>) -> Self {
    let channels = channels.into_iter().map(|c| format!("#{}", c)).collect();

    let config = Config {
      nickname: Some("justinfan12345".to_owned()),
      server: Some("irc.chat.twitch.tv".to_owned()),
      port: Some(6667),
      use_tls: Some(false),
      channels,
      ..Default::default()
    };

    let client = Client::from_config(config)
      .await
      .expect("Could not create IRC client");

    Self { client }
  }

  pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
    self.client.identify()?;
    let mut stream = self.client.stream()?;
    while let Some(message) = stream.next().await.transpose()? {
      let Command::PRIVMSG(ref channel, ref msg) = message.command else {
        continue;
      };
      println!(
        "[{}] {}: {}",
        channel,
        message.source_nickname().unwrap_or("unknown"),
        msg
      );
    }

    Ok(())
  }
}
