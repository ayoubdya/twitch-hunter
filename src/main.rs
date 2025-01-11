use ::irc::proto::chan;
use lazy_static::lazy_static;
use regex::Regex;
use std::error::Error;
use tmi;
use tokio::sync::mpsc::{channel, Sender};

mod helix;
use helix::TwitchHelix;

mod irc;
use irc::TwitchIrc;

struct TwitchClient {
  channel: String,
  client: tmi::Client,
  sender: Sender<String>,
}

impl TwitchClient {
  async fn new(sender: Sender<String>, channel: String) -> Result<Self, Box<dyn Error>> {
    let mut client = tmi::Client::anonymous().await.map_err(|err| {
      eprintln!(
        "ERROR: COULD NOT CREATE ANONYMOUS TWITCH CLIENT FOR {channel}: {:?}",
        err
      );
      err
    })?;
    let channel = format!("#{}", channel);
    client.join(&channel).await?;
    Ok(Self {
      channel,
      client,
      sender,
    })
  }

  async fn reconnect(&mut self) -> Result<(), Box<dyn Error>> {
    self.client.reconnect().await?;
    self.client.join(&self.channel).await?;
    Ok(())
  }

  async fn run(&mut self) -> Result<(), Box<dyn Error>> {
    loop {
      let Ok(msg) = self.client.recv().await else {
        eprintln!("ERROR: COULD NOT RECEIVE IRC MESSAGE");
        self.reconnect().await?;
        continue;
      };

      let msg = msg.as_typed().map_err(|err| {
        eprintln!("ERROR: COULD NOT PARSE IRC MESSAGE: {:?}", err);
        err
      })?;

      match msg {
        tmi::Message::Privmsg(msg) => {
          let text = msg.text();
          if REGEX_FILTER.captures(text).is_some() {
            let msg = format!("{}| {}: {}", self.channel, msg.sender().name(), text);
            self.sender.send(msg).await?;
          }
        }
        tmi::Message::Reconnect => {
          self.client.reconnect().await.map_err(|err| {
            eprintln!("ERROR: COULD NOT RECONNECT TO THE CHANNEL: {:?}", err);
            err
          })?;
          self.client.join(&self.channel).await.map_err(|err| {
            eprintln!("ERROR: COULD NOT REJOIN THE CHANNEL: {:?}", err);
            err
          })?;
        }
        _ => {}
      }
    }
  }
}

lazy_static! {
  static ref REGEX_FILTER: Regex = Regex::new(r"https://.+").unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let client_id = std::env::var("TWITCH_CLIENT_ID").expect("TWITCH_CLIENT_ID not set");
  let access_token = std::env::var("TWITCH_ACCESS_TOKEN").expect("TWITCH_ACCESS_TOKEN not set");
  let helix = TwitchHelix::new(client_id, access_token);

  // let categories = helix.get_categories("Rust".to_string()).await?;
  // let category_id = categories.first().unwrap().id.clone();

  let category_id = "263490";

  let streams = helix.get_streams(category_id).await?;
  println!("Found {} streams", streams.len());

  // const CHANNELS: [&str; 10] = [
  //   "microg0d",
  //   "oilrats",
  //   "loltyler1",
  //   "sodapoppin",
  //   "nmplol",
  //   "timthetatman",
  //   "piratesoftware",
  //   "northernlion",
  //   "lirik",
  //   "agent00",
  // ];

  // let channels: Vec<String> = streams.into_iter().map(|s| s.user_login).collect();
  // let mut irc = TwitchIrc::new(channels).await;
  // irc.run().await?;

  let (tx, mut rx) = channel(1000);

  for channel in streams.into_iter() {
    let tx = tx.clone();
    tokio::spawn(async move {
      let mut client = match TwitchClient::new(tx, channel.user_login).await {
        Ok(client) => client,
        Err(err) => {
          eprintln!("ERROR: COULD NOT CREATE TWITCH CLIENT: {:?}", err);
          return;
        }
      };
      client.run().await.unwrap_or_else(|err| {
        eprintln!("ERROR: COULD NOT RUN TWITCH CLIENT: {:?}", err);
      });
    });
  }

  while let Some(msg) = rx.recv().await {
    println!("{}", msg);
  }

  Ok(())
}

// use futures_util::StreamExt;
// use irc::client::prelude::*;

// let irc_config = Config {
//   server: Some("irc.chat.twitch.tv".to_owned()),
//   channels: vec!["#oilrats".to_owned()],
//   nickname: Some("microg0d".to_owned()),
//   password: Some("jqn6zww5u79096hyajv4bhlqr2v90d".to_owned()),
//   port: Some(6697),
//   ping_time: Some(10),
//   ping_timeout: Some(10),
//   use_tls: Some(false),
//   ..Config::default()
// };

// let mut irc_client = Client::from_config(irc_config).await.unwrap();
// irc_client.identify().unwrap();

// let mut stream = irc_client.stream().unwrap();

// while let Some(message) = stream.next().await.transpose().unwrap() {
//   println!("{message:?}");
//   // if let Command::PRIVMSG(_, msg) = message.command {
//   // }
// }
