use serde::Deserialize;
use std::time::{Duration, Instant};

use super::data::*;
use super::errors::*;

#[derive(Deserialize, Debug)]
pub struct AccessTokenJSON {
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

pub fn convert_token(token : AccessTokenJSON) -> Result<AccessToken, Error> {
  if token.expires_in < 0 {
    return Err( Error::from( format!("received expires_in field in AccessToken is negative!: {}", token.expires_in ) ) );
  }

  let expires_at  = Instant::now() + Duration::from_secs( token.expires_in.try_into().unwrap() );

  Ok( AccessToken{ access_token : token.access_token, refresh_token : token.refresh_token, expires_at } )
}

pub async fn apply_timeout_and_send(mut build : reqwest::RequestBuilder, timeout : &Option<Duration>) -> Result<reqwest::Response, Error>
{
    if let Some( d ) = timeout {
        build = build.timeout(*d);
    }
    let res = build.send().await?;
    if ! res.status().is_success()  {
        return Err( Error::from( format!("Failed to send request. status: {} tex: {}", res.status(), res.text().await? ) ) );
    }
    Ok( res )
}



pub async fn get_client_access_token(client : &reqwest::Client, cfg : &ConnectConfig, timeout : &Option<Duration>)
  -> Result<AccessToken, Error> {

  let params = [("grant_type", "password"),
                          ("client_id", &cfg.client_id),
                          ("client_secret", &cfg.client_secret),
                          ("username", &cfg.username),
                          ("password", &cfg.password),
                          ("scope", "read_station read_homecoach")];

  let build = client.post("https://api.netatmo.com/oauth2/token").form(&params);

  let res = apply_timeout_and_send(build, timeout).await?;

  let res = res.json::<AccessTokenJSON>().await?;

  convert_token(res)
}

pub async fn get_stations_data(client : &reqwest::Client, token : &AccessToken, timeout : &Option<Duration>)
  -> Result<StationsData, Error> {

  //let params = [("device_id", "04255185")];
  let build = client.get("https://api.netatmo.com/api/getstationsdata")
    .header("Authorization", String::from("Bearer ") + &token.access_token);

  let res = apply_timeout_and_send(build, timeout).await?;

   let res = res.json::<StationsData>().await?;

   Ok( res )
}

pub async fn get_fresh_token(client : &reqwest::Client, cfg : &ConnectConfig, old_token: &AccessToken, timeout : &Option<Duration>)
    -> Result<AccessToken, Error> {

  let params = [("grant_type", "refresh_token"),
                          ("refresh_token", &old_token.refresh_token),
                          ("client_id", &cfg.client_id),
                          ("client_secret", &cfg.client_secret)];

  let build = client.post("https://api.netatmo.com/oauth2/token").form(&params);

  let res = apply_timeout_and_send(build, timeout).await?;

  let res = res.json::<AccessTokenJSON>().await?;

  convert_token( res )
}

pub async fn get_homecoachs_data(client : &reqwest::Client, token : &AccessToken, timeout : &Option<Duration>)
  -> Result<HomeCoachsData, Error>
{
    let build = client.get("https://api.netatmo.com/api/gethomecoachsdata")
        .header("Authorization", String::from("Bearer ") + &token.access_token)
        .header("accept", "application/json");

    let res = apply_timeout_and_send(build, timeout).await?;

    let res = res.json::<HomeCoachsData>().await?;

    Ok( res )
}
