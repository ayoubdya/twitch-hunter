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

#[tokio::main]
async fn main() {
  const SAVE_FILE: &str = "creds.json";

  let args = Args::parse();

  let creds = match (args.client_id, args.access_token) {
    (Some(client_id), Some(access_token)) => Credentials {
      client_id,
      access_token,
    },
    _ => {
      let file = File::open(SAVE_FILE);
      match file {
        Ok(file) => serde_json::from_reader(file).unwrap_or_else(|err| {
          eprintln!("ERROR: Could not parse credentials from file {SAVE_FILE} : {err}");
          exit(1);
        }),
        Err(err) => {
          eprintln!("ERROR: Missing credentials from arguments or file {SAVE_FILE} : {err}");
          exit(1);
        }
      }
    }
  };
  let helix = TwitchHelix::new(&creds);

  if args.save {
    let file = File::create(SAVE_FILE).unwrap_or_else(|err| {
      eprintln!("ERROR: Could not create file {SAVE_FILE} : {err}");
      exit(1);
    });
    if let Err(err) = serde_json::to_writer(file, &creds) {
      eprintln!("ERROR: Could not write credentials to file {SAVE_FILE} : {err}");
      exit(1);
    }
  }

  let streams = match (args.category_name, args.streams) {
    (Some(name), None) => {
      let category_id = match helix.get_category_id(name.as_str()).await {
        Ok(Some(id)) => id,
        Ok(None) => {
          eprintln!("ERROR: Category not found");
          exit(1);
        }
        Err(err) => {
          eprintln!("ERROR: {err}");
          exit(1);
        }
      };

      let streams = helix
        .get_streams(category_id.as_str())
        .await
        .unwrap_or_else(|err| {
          eprintln!("ERROR: Could not get streams : {err}");
          exit(1);
        });
      println!("Found {} streams", streams.len());

      streams.into_iter().map(|s| s.user_login).collect()
    }
    (None, Some(streams)) => streams,
    _ => {
      eprintln!("ERROR: Bad arguments combination");
      exit(1);
    }
  };

  // Marvel Rivals 1264310518
  // let category_id = "1264310518".to_owned();

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
        eprintln!("ERROR: Could not run IRC client : {err}",);
        exit(1);
      });
    });
  }

  while let Some(msg) = rx.recv().await {
    println!("{msg}");
  }
}
