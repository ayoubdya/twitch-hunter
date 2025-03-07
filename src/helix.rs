use reqwest::{header, Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Deserialize)]
pub struct Category {
  pub id: String,
  pub name: String,
  // pub box_art_url: String,
}

#[derive(Deserialize)]
pub struct User {
  // pub id: String,
  pub login: String,
  // pub display_name: String,
  // #[serde(rename = "type")]
  // pub type_: String,
  // pub broadcaster_type: String,
  // pub description: String,
  // pub profile_image_url: String,
  // pub offline_image_url: String,
  // pub view_count: i64,
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

#[derive(Deserialize)]
struct ResponseNoPag<T> {
  data: Vec<T>,
}

#[derive(Deserialize, Serialize)]
pub struct Credentials {
  pub client_id: String,
  pub access_token: String,
}

pub struct TwitchHelix {
  client: Client,
  // client_id: String,
  // access_token: String,
}

impl TwitchHelix {
  const BASE_URL: &str = "https://api.twitch.tv/helix";

  pub fn new(creds: &Credentials) -> Self {
    let mut headers = header::HeaderMap::new();
    headers.insert(
      "Client-ID",
      header::HeaderValue::from_str(creds.client_id.as_str()).unwrap(),
    );
    headers.insert(
      "Authorization",
      header::HeaderValue::from_str(&format!("Bearer {}", creds.access_token)).unwrap(),
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
      let mut request = self
        .client
        .get(format!("{}/streams", Self::BASE_URL))
        .query(&[("type", "live"), ("first", "100"), ("game_id", category_id)]);

      if let Some(cursor) = cursor {
        request = request.query(&[("after", cursor)]);
      }

      let res = request.send().await?;

      match res.status() {
        StatusCode::OK => (),
        StatusCode::BAD_REQUEST => {
          let body = res.text().await?;
          eprintln!("Bad request error: {}", body);
          return Err("Bad request".into());
        }
        StatusCode::UNAUTHORIZED => {
          let body = res.text().await?;
          eprintln!("Unauthorized error: {}", body);
          return Err("Unauthorized".into());
        }
        status => {
          let body = res.text().await?;
          eprintln!("Unexpected status code {}: {}", status, body);
          return Err("Unexpected error".into());
        }
      }

      let body: Response<Stream> = res.json().await?;
      streams.extend(body.data);

      if let Some(new_cursor) = body.pagination.cursor {
        cursor = Some(new_cursor);
      } else {
        break;
      }
    }

    Ok(streams)
  }

  pub async fn get_categories(&self, keyword: &str) -> Result<Vec<Category>, Box<dyn Error>> {
    let url = format!("{}/search/categories?query={}", Self::BASE_URL, keyword);

    let res = self.client.get(url).send().await?;

    match res.status() {
      StatusCode::OK => (),
      StatusCode::BAD_REQUEST => {
        eprintln!("ERROR: bad request: {:?}", res.text().await?);
        return Err("Bad request".into());
      }
      StatusCode::UNAUTHORIZED => {
        eprintln!("ERROR: unauthorized: {:?}", res.text().await?);
        return Err("Unauthorized".into());
      }
      _ => {
        eprintln!("ERROR: could not get categories: {:?}", res.text().await?);
        return Err("Unexpected error".into());
      }
    }

    let body: Response<Category> = res.json().await?;

    Ok(body.data)
  }

  pub async fn get_users(
    &self,
    streams: Vec<String>,
  ) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
    let streams = streams
      .into_iter()
      .map(|s| s.to_lowercase())
      .collect::<Vec<String>>();

    let url = format!("{}/users?login={}", Self::BASE_URL, streams.join("&login="));

    let res = self.client.get(url).send().await?;

    match res.status() {
      StatusCode::OK => (),
      StatusCode::BAD_REQUEST => {
        eprintln!("ERROR: bad request: {:?}", res.text().await?);
        return Err("Bad request".into());
      }
      StatusCode::UNAUTHORIZED => {
        eprintln!("ERROR: unauthorized: {:?}", res.text().await?);
        return Err("Unauthorized".into());
      }
      _ => {
        eprintln!("ERROR: could not get users: {:?}", res.text().await?);
        return Err("Unexpected error".into());
      }
    }

    let body: ResponseNoPag<User> = res.json().await?;

    let good = body
      .data
      .into_iter()
      .map(|u| u.login)
      .collect::<Vec<String>>();
    let bad = streams.into_iter().filter(|s| !good.contains(s)).collect();
    Ok((good, bad))
  }

  pub async fn get_category_id(&self, keyword: &str) -> Result<Option<String>, Box<dyn Error>> {
    let categories = self.get_categories(keyword).await?;
    Ok(
      categories
        .into_iter()
        .find(|c| c.name.eq_ignore_ascii_case(keyword))
        .map(|c| c.id),
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn new_client() -> TwitchHelix {
    let client_id = std::env::var("TWITCH_CLIENT_ID").expect("TWITCH_CLIENT_ID not set");
    let access_token = std::env::var("TWITCH_ACCESS_TOKEN").expect("TWITCH_ACCESS_TOKEN not set");
    let creds = Credentials {
      client_id,
      access_token,
    };

    TwitchHelix::new(&creds)
  }

  #[tokio::test]
  async fn test_get_users() {
    let helix = new_client();

    let streams = vec!["loltyler1".to_owned(), "fsdqfqsdfsdfsqdfsqdf".to_owned()];

    let (good, bad) = helix.get_users(streams.clone()).await.unwrap();
    assert_eq!(good.len(), 1);
    assert_eq!(bad.len(), 1);

    let (good, bad) = helix
      .get_users(streams.iter().map(|s| s.to_uppercase()).collect())
      .await
      .unwrap();
    assert_eq!(good.len(), 1);
    assert_eq!(bad.len(), 1);
  }

  #[tokio::test]
  async fn test_get_categories() {
    let helix = new_client();

    let data = helix.get_categories("RUsT").await.unwrap();
    assert!(data.len() > 0);

    let data = helix.get_categories("sqdfsdfqqsdf").await.unwrap();
    assert_eq!(data.len(), 0);
  }

  #[tokio::test]
  async fn test_get_category_id() {
    let helix = new_client();

    let category_id = helix.get_category_id("RUsT").await.unwrap();
    assert_eq!(category_id, Some("263490".to_string()));

    let category_id = helix.get_category_id("sqdfsdfqqsdf").await.unwrap();
    assert_eq!(category_id, None);
  }

  #[tokio::test]
  async fn test_get_streams() {
    let helix = new_client();

    let body = helix.get_streams("263490").await.unwrap(); //177157840 33214
    assert!(body.len() > 0);

    let body = helix.get_streams("121515155151").await.unwrap();
    assert_eq!(body.len(), 0);
  }
}
