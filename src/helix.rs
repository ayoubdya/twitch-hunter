use reqwest::{header, Client, StatusCode};
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize)]
pub struct Category {
  pub id: String,
  pub name: String,
  // pub box_art_url: String,
}

#[derive(Deserialize)]
pub struct Stream {
  pub user_login: String,
  // pub id: String,
  // pub user_id: String,
  // pub user_name: String,
  // pub game_id: String,
  // pub game_name: String,
  // #[serde(rename = "type")]
  // pub type_: String,
  // pub title: String,
  // pub viewer_count: i64,
  // pub started_at: String,
}

#[derive(Deserialize)]
struct Pagination {
  cursor: Option<String>,
}

#[derive(Deserialize)]
struct Response<T> {
  data: Vec<T>,
  pagination: Pagination,
}
pub struct TwitchHelix {
  client: Client,
  // client_id: String,
  // access_token: String,
}

impl TwitchHelix {
  pub fn new(client_id: String, access_token: String) -> Self {
    let mut headers = header::HeaderMap::new();
    headers.insert(
      "Client-ID",
      header::HeaderValue::from_str(&client_id).unwrap(),
    );
    headers.insert(
      "Authorization",
      header::HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap(),
    );
    let client = Client::builder()
      .default_headers(headers)
      .build()
      .expect("Could not create reqwest client");

    Self {
      client,
      // client_id,
      // access_token,
    }
  }

  pub async fn get_streams(&self, category_id: &str) -> Result<Vec<Stream>, Box<dyn Error>> {
    let mut streams = Vec::new();
    let mut cursor = None;
    loop {
      let (new_streams, new_cursor) = self.get_streams_internal(category_id, cursor).await?;
      streams.extend(new_streams);
      cursor = new_cursor;
      if cursor.is_none() {
        break;
      }
    }
    Ok(streams)
  }

  async fn get_streams_internal(
    &self,
    category_id: &str,
    after_cursor: Option<String>,
  ) -> Result<(Vec<Stream>, Option<String>), Box<dyn Error>> {
    let url = format!(
      "https://api.twitch.tv/helix/streams?game_id={}&type=live&first=100&after={}",
      category_id,
      after_cursor.unwrap_or("".to_string())
    );

    let res = self.client.get(&url).send().await?;

    match res.status() {
      StatusCode::OK => (),
      StatusCode::BAD_REQUEST => {
        eprintln!("ERROR: BAD REQUEST: {:?}", res.text().await?);
        return Err("Bad request".into());
      }
      StatusCode::UNAUTHORIZED => {
        eprintln!("ERROR: UNAUTHORIZED: {:?}", res.text().await?);
        return Err("Unauthorized".into());
      }
      _ => {
        eprintln!("ERROR: COULD NOT GET STREAMS: {:?}", res.text().await?);
        return Err("Unexpected error".into());
      }
    }

    let body: Response<Stream> = res.json::<_>().await?;

    return Ok((body.data, body.pagination.cursor));
  }

  pub async fn get_categories(&self, keyword: &str) -> Result<Vec<Category>, Box<dyn Error>> {
    let url = format!(
      "https://api.twitch.tv/helix/search/categories?query={}",
      keyword
    );

    let res = self.client.get(&url).send().await?;

    match res.status() {
      StatusCode::OK => (),
      StatusCode::BAD_REQUEST => {
        eprintln!("ERROR: BAD REQUEST: {:?}", res.text().await?);
        return Err("Bad request".into());
      }
      StatusCode::UNAUTHORIZED => {
        eprintln!("ERROR: UNAUTHORIZED: {:?}", res.text().await?);
        return Err("Unauthorized".into());
      }
      _ => {
        eprintln!("ERROR: COULD NOT GET CATEGORIES: {:?}", res.text().await?);
        return Err("Unexpected error".into());
      }
    }

    let body: Response<Category> = res.json::<_>().await?;

    Ok(body.data)
  }

  pub async fn get_category_id(&self, keyword: &str) -> Result<Option<String>, Box<dyn Error>> {
    let categories = self.get_categories(keyword).await?;
    let keyword = keyword.to_lowercase();

    Ok(
      categories
        .into_iter()
        .filter(|c| c.name.to_lowercase() == keyword)
        .next()
        .map(|c| c.id),
    )
  }
}

mod tests {
  #[cfg(test)]
  use super::*;

  #[tokio::test]
  async fn test_get_categories() {
    let client_id = std::env::var("TWITCH_CLIENT_ID").expect("TWITCH_CLIENT_ID not set");
    let access_token = std::env::var("TWITCH_ACCESS_TOKEN").expect("TWITCH_ACCESS_TOKEN not set");

    let helix = TwitchHelix::new(client_id, access_token);
    let data = helix.get_categories("Rust").await.unwrap();
    assert!(data.len() > 0);
  }

  #[tokio::test]
  async fn test_get_category_id() {
    let client_id = std::env::var("TWITCH_CLIENT_ID").expect("TWITCH_CLIENT_ID not set");
    let access_token = std::env::var("TWITCH_ACCESS_TOKEN").expect("TWITCH_ACCESS_TOKEN not set");

    let helix = TwitchHelix::new(client_id, access_token);

    let category_id = helix.get_category_id("Rust").await.unwrap();
    assert_eq!(category_id, Some("263490".to_string()));

    let category_id = helix.get_category_id("sqdfsdfqqsdf").await.unwrap();
    assert_eq!(category_id, None);
  }

  #[tokio::test]
  async fn test_get_streams() {
    let client_id = std::env::var("TWITCH_CLIENT_ID").expect("TWITCH_CLIENT_ID not set");
    let access_token = std::env::var("TWITCH_ACCESS_TOKEN").expect("TWITCH_ACCESS_TOKEN not set");

    let helix = TwitchHelix::new(client_id, access_token);
    let body = helix.get_streams("263490").await.unwrap(); //177157840 33214
    assert!(body.len() > 0);
  }
}
