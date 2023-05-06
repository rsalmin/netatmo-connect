use reqwest;
use serde::{Serialize, Deserialize};
use confy;
use std::time::{Duration, Instant};

use super::data::*;

#[derive(Debug)]
pub struct Error {
  pub msg : String,
}

impl From<Error> for String {
  fn from(e : Error) -> String {
    e.msg
  }
}

impl From<reqwest::Error> for Error {
  fn from(e : reqwest::Error) -> Error {
    Error { msg : format!("{:?}", e) }
  }
}

impl From<String> for Error {
  fn from(e : String) -> Error {
    Error { msg : e }
  }
}

impl From<confy::ConfyError> for Error {
  fn from(e : confy::ConfyError) -> Error {
    Error { msg : format!("{:?}", e) }
  }
}

#[derive(Deserialize, Debug)]
struct AccessTokenJSON {
  access_token : String,
  refresh_token : String,
  expires_in : i32,
}

#[derive(Debug)]
pub struct AccessToken {
  access_token : String,
  refresh_token : String,
  pub expires_at : Instant,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConnectConfig {
    client_id: String,
    client_secret: String,
    username: String,
    password: String,
}

/// Default for initial save ;)
impl ::std::default::Default for ConnectConfig  {
    fn default() -> Self {
     Self { client_id: "cliend_id".into(), client_secret : "client_secret".into(),
                username : "username".into(), password : "password".into() }
    }
}

fn convert_token(token : AccessTokenJSON) -> Result<AccessToken, Error> {
  if token.expires_in < 0 {
    return Err( Error::from( format!("received expires_in field in AccessToken is negative!: {}", token.expires_in ) ) );
  }

  let expires_at  = Instant::now() + Duration::from_secs( token.expires_in.try_into().unwrap() );

  Ok( AccessToken{ access_token : token.access_token, refresh_token : token.refresh_token, expires_at } )
}

pub async fn get_access_token(client : &reqwest::Client, cfg : &ConnectConfig)
  -> Result<AccessToken, Error> {

  let params = [("grant_type", "password"),
                          ("client_id", &cfg.client_id),
                          ("client_secret", &cfg.client_secret),
                          ("username", &cfg.username),
                          ("password", &cfg.password)];

  let res = client.post("https://api.netatmo.com/oauth2/token").form(&params).send().await?;

  if ! res.status().is_success()  {
    return Err( Error::from( format!("unsuccesseful status: {}", res.status() ) ) );
  }

  let res = res.json::<AccessTokenJSON>().await?;

  convert_token(res)
}

pub async fn get_stations_data(client : &reqwest::Client, token : &AccessToken)
  -> Result<StationsData, Error> {

  //let params = [("device_id", "04255185")];
  let res = client.get("https://api.netatmo.com/api/getstationsdata")
    .header("Authorization", String::from("Bearer ") + &token.access_token)
    .send().await?;

  if ! res.status().is_success()  {
    return Err( Error::from( format!("unsuccesseful status: {}", res.status() ) ) );
  }

   let res = res.json::<StationsData>().await?;

   Ok( res )
}

pub async fn get_fresh_token(client : &reqwest::Client, cfg : &ConnectConfig, old_token: &AccessToken)
    -> Result<AccessToken, Error> {

  let params = [("grant_type", "refresh_token"),
                          ("refresh_token", &old_token.refresh_token),
                          ("client_id", &cfg.client_id),
                          ("client_secret", &cfg.client_secret)];

  let res = client.post("https://api.netatmo.com/oauth2/token").form(&params).send().await?;

  if ! res.status().is_success()  {
    return Err( Error::from( format!("unsuccesseful status: {}", res.status() ) ) );
  }

  let res = res.json::<AccessTokenJSON>().await?;

  convert_token( res )
}