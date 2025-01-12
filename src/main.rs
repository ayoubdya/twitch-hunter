use clap::Parser;
use std::{process::exit, sync::Arc};
use tokio::sync::mpsc::channel;

mod helix;
use helix::TwitchHelix;

mod irc;
use irc::TwitchIrc;

mod args;
use args::Args;

#[tokio::main]
async fn main() {
  let args = Args::parse();

  let helix = TwitchHelix::new(args.client_id, args.access_token);

  let category_id = match helix.get_category_id(args.category_name.as_str()).await {
    Ok(Some(id)) => id,
    Ok(None) => {
      eprintln!("ERROR: Category not found");
      exit(1);
    }
    Err(e) => {
      eprintln!("ERROR: {}", e);
      exit(1);
    }
  };

  // Marvel Rivals 1264310518
  // let category_id = "1264310518".to_owned();

  let Ok(streams) = helix.get_streams(category_id.as_str()).await else {
    eprintln!("ERROR: Could not get streams");
    exit(1);
  };
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
      let Ok(_) = irc.run().await else {
        eprintln!("ERROR: Could not run IRC client");
        exit(1);
      };
    });
  }

  while let Some(msg) = rx.recv().await {
    println!("{} | {}: {}", msg.channel, msg.nickname, msg.msg);
  }
}
