use futures_util::stream::StreamExt;
use irc::client::prelude::*;
use regex::Regex;
use std::{error::Error, fmt::Display, sync::Arc};
use tokio::sync::mpsc::Sender;

pub struct Message {
  pub channel: String,
  pub nickname: String,
  pub msg: String,
  // pub captures: Vec<String>,
}

impl Display for Message {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} | {}: {}", self.channel, self.nickname, self.msg)
  }
}

pub struct TwitchIrc {
  pub sender: Sender<Message>,
  pub client: Client,
  pub regex_filter: Arc<Regex>,
}

impl TwitchIrc {
  pub async fn new(
    sender: Sender<Message>,
    channels: Vec<String>,
    regex_filter: Arc<Regex>,
  ) -> Self {
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

    Self {
      sender,
      regex_filter,
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
      if let Some(_) = self.regex_filter.captures(msg) {
        let nickname = message.source_nickname().unwrap_or("unknown").to_owned();

        // let captures = captures
        //   .iter()
        //   .skip(1)
        //   .filter_map(|c| c)
        //   .map(|c| c.as_str().to_owned())
        //   .collect::<Vec<String>>();

        let msg = Message {
          channel: channel.to_owned(),
          nickname,
          msg: msg.to_owned(),
          // captures,
        };
        self.sender.send(msg).await?;
      }
    }

    Ok(())
  }
}
