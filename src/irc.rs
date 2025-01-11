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
  pub channel: String,
  pub client: Client,
}

impl TwitchIrc {
  pub async fn new(sender: Sender<String>, channel: String) -> Self {
    // let channels = channels.into_iter().map(|c| format!("#{}", c)).collect();
    let channels = vec![format!("#{}", channel)];

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

    Self {
      sender,
      channel,
      client,
    }
  }

  pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
    self.client.identify()?;
    let mut stream = self.client.stream()?;
    println!("Connected to Twitch IRC channel {}", self.channel);
    while let Some(message) = stream.next().await.transpose()? {
      let Command::PRIVMSG(_, ref msg) = message.command else {
        continue;
      };
      if REGEX_FILTER.captures(msg).is_some() {
        let msg = format!(
          "{}| {}: {}",
          self.channel,
          message.source_nickname().unwrap_or("unknown"),
          msg
        );
        self.sender.send(msg).await?;
      }
    }

    Ok(())
  }
}
