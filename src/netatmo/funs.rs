use reqwest;
use serde::{Serialize, Deserialize};
use confy;

use super::data::*;

#[derive(Debug)]
pub struct Error {
  pub msg : String,
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
pub struct AccessToken {
  access_token : String,
  refresh_token : String,
  expires_in : i32,
}

#[derive(Serialize, Deserialize)]
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

  let res = res.json::<AccessToken>().await?;

  return Ok( res );
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
