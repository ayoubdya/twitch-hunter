use futures::StreamExt;
use irc::client::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;
use std::error::Error;
use tokio::sync::mpsc::Sender;

lazy_static! {
  pub static ref REGEX_FILTER: Regex = Regex::new(r"https://.+").unwrap();
}

pub struct TwitchIrc {
  pub sender: Sender<String>,
  pub channels: Vec<String>,
  pub client: Client,
}

impl TwitchIrc {
  pub async fn new(sender: Sender<String>, channels: Vec<String>) -> Self {
    let channels_hash = channels.iter().map(|c| format!("#{}", c)).collect();
    // let channels = vec![format!("#{}", channel)];

    let config = Config {
      nickname: Some("justinfan12345".to_owned()),
      server: Some("irc.chat.twitch.tv".to_owned()),
      port: Some(6667),
      use_tls: Some(false),
      channels: channels_hash,
      ..Default::default()
    };

    let client = Client::from_config(config)
      .await
      .expect("Could not create IRC client");

    Self {
      sender,
      channels,
      client,
    }
  }

  pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
    self.client.identify()?;
    let mut stream = self.client.stream()?;
    while let Some(message) = stream.next().await.transpose()? {
      let Command::PRIVMSG(ref channel, ref msg) = message.command else {
        continue;
      };
      if REGEX_FILTER.captures(msg).is_some() {
        let msg = format!(
          "{}| {}: {}",
          channel,
          message.source_nickname().unwrap_or("unknown"),
          msg
        );
        self.sender.send(msg).await?;
      }
    }

    Ok(())
  }
}
