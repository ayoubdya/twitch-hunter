use clap::Parser;
use serde_json;
use std::{fs::File, process::exit, sync::Arc};
use tokio::sync::mpsc::channel;

mod helix;
use helix::{Credentials, TwitchHelix};

mod irc;
use irc::TwitchIrc;

mod args;
use args::Args;

mod utils;

#[tokio::main]
async fn main() {
  const SAVE_FILENAME: &str = "creds.json";
  let save_path = std::env::current_exe()
    .unwrap_or_else(|err| {
      exit!("ERROR: Could not get current executable path : {err}");
    })
    .with_file_name(SAVE_FILENAME);

  let args = Args::parse();

  let creds = match (args.client_id, args.access_token) {
    (Some(client_id), Some(access_token)) => Credentials {
      client_id,
      access_token,
    },
    _ => {
      let file = File::open(&save_path);
      match file {
        Ok(file) => serde_json::from_reader(file).unwrap_or_else(|err| {
          exit!("ERROR: Could not parse credentials from file {SAVE_FILENAME} : {err}");
        }),
        Err(err) => {
          exit!("ERROR: Missing credentials from arguments or file {SAVE_FILENAME} : {err}");
        }
      }
    }
  };
  let helix = TwitchHelix::new(&creds);

  if args.save {
    let file = File::create(save_path).unwrap_or_else(|err| {
      exit!("ERROR: Could not create file {SAVE_FILENAME} : {err}");
    });
    if let Err(err) = serde_json::to_writer(file, &creds) {
      exit!("ERROR: Could not write credentials to file {SAVE_FILENAME} : {err}");
    }
  }

  let streams = match (args.category_name, args.streams) {
    (Some(name), None) => {
      let category_id = match helix.get_category_id(name.as_str()).await {
        Ok(Some(id)) => id,
        Ok(None) => {
          exit!("ERROR: Category not found");
        }
        Err(err) => {
          exit!("ERROR: {err}");
        }
      };

      println!("Getting streams for category {} ...", name);

      let streams = helix
        .get_streams(category_id.as_str())
        .await
        .unwrap_or_else(|err| {
          exit!("ERROR: Could not get streams : {err}");
        });

      println!("Found {} streams", streams.len());

      streams.into_iter().map(|s| s.user_login).collect()
    }
    (None, Some(streams)) => {
      let (good, bad) = helix.get_users(streams).await.unwrap_or_else(|err| {
        exit!("ERROR: Could not get users : {err}");
      });

      if !bad.is_empty() {
        println!("These streams were not found: {}", bad.join(", "));
      }

      if good.is_empty() {
        exit!("ERROR: No streams found");
      }

      good
    }
    _ => {
      exit!("ERROR: Bad arguments combination");
    }
  };

  let (tx, mut rx) = channel(100);

  let regex_filter = Arc::new(args.filter);

  let batch_size = args.batch_size;
  for i in 0..streams.len() / batch_size + 1 {
    // println!("Spawning batch {}", i);

    let tx = tx.clone();
    let regex_filter = regex_filter.clone();

    let streams_batch = streams
      .iter()
      .skip(i * batch_size)
      .take(batch_size)
      .map(|s| s.clone())
      .collect();

    tokio::spawn(async move {
      let mut irc = TwitchIrc::new(tx, streams_batch, regex_filter).await;
      irc.run().await.unwrap_or_else(|err| {
        exit!("ERROR: Could not run IRC client : {err}");
      });
    });
  }

  while let Some(msg) = rx.recv().await {
    println!("{msg}");
  }
}
