use reqwest;
use serde::{Serialize, Deserialize};
use confy;
use std::time::{Duration, Instant};
use url;
use webbrowser;
use actix_web::middleware::Logger;
use actix_web::{get, web, App, HttpServer};
use actix_web::{HttpResponse, Responder};
use std::cell::RefCell;
use std::sync::{RwLock,Mutex,Arc};
use log;

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

impl From<url::ParseError> for Error {
  fn from(e : url::ParseError) -> Error {
    Error { msg : format!("{:?}", e) }
  }
}

impl From<std::io::Error> for Error {
  fn from(e : std::io::Error) -> Error {
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

impl<T> From<std::sync::PoisonError<T>> for Error {
  fn from(e : std::sync::PoisonError<T>) -> Error {
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
    arbitrary_but_unique_string: String,
}

/// Default for initial save ;)
impl ::std::default::Default for ConnectConfig  {
    fn default() -> Self {
     Self { client_id: "cliend_id".into(), client_secret : "client_secret".into(),
                username : "username".into(), password : "password".into(),
                arbitrary_but_unique_string : "arbitrary_not_so_unique_string".into(), }
    }
}

fn convert_token(token : AccessTokenJSON) -> Result<AccessToken, Error> {
  if token.expires_in < 0 {
    return Err( Error::from( format!("received expires_in field in AccessToken is negative!: {}", token.expires_in ) ) );
  }

  let expires_at  = Instant::now() + Duration::from_secs( token.expires_in.try_into().unwrap() );

  Ok( AccessToken{ access_token : token.access_token, refresh_token : token.refresh_token, expires_at } )
}

async fn apply_timeout_and_send(mut build : reqwest::RequestBuilder, timeout : &Option<Duration>) -> Result<reqwest::Response, Error>
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


#[derive(Deserialize)]
struct AuthRespParams {
    state: String,
    code: String,
}

#[derive(Clone)]
struct SharedWebData {
    state: String,
    code: String
}

#[get("/api/authorization_response")]
async fn authorization_response_handler(params : web::Query<AuthRespParams>, data : web::Data<Arc<RwLock<SharedWebData>>>) -> impl Responder {

    println!("Helo I am here! ");

    if let Ok( data ) = data.read() {
        if data.state != params.state {
            return HttpResponse::Unauthorized().reason("state string is not matching").finish();
        }
    }
    else {
        return HttpResponse::InternalServerError().reason("programmer fuckuped mutex").finish();
    }

    if let Ok( mut data ) = data.write() {
        data.code = params.code.clone();
    } else {
        return HttpResponse::InternalServerError().reason("programmer fuckuped mutex").finish();
    }

    HttpResponse::Ok().finish()
}

#[tokio::main]
async fn wait_for_response_web_task(data : Arc<RwLock<SharedWebData>>)
{
    match wait_for_response_web_task_pure(data).await {
        Err ( e ) => println!("Error {:?}", e),
        Ok( _ ) => println!("Ok"),
    }
}

async fn wait_for_response_web_task_pure(data : Arc<RwLock<SharedWebData>>) -> Result<String, Error>
{

    let data2 = data.clone();
    let server = HttpServer::new(  move || {
       App::new()
            .service(authorization_response_handler)
            .app_data( web::Data::new( data2.clone() ) )
    })
    .workers(1)
    .bind(("127.0.0.1", 8000))?;

    let () = server.run().await?;

    println!("Server finished data.code = {}", data.read()?.code);

    Ok(String::new())
}

pub async fn authorize(client : &reqwest::Client, cfg : &ConnectConfig, timeout : &Option<Duration>)
  -> Result<(), Error> {

    let params = [("client_id", &cfg.client_id),
                          ("redirect_uri", &String::from("http://localhost:8000/api/authorization_response")),
                          ("scope", &String::from("read_station read_homecoach")),
                          ("state", &cfg.arbitrary_but_unique_string),
                         ];

    let url = reqwest::Url::parse_with_params("https://api.netatmo.com/oauth2/authorize", &params)?;

    let data = Arc::new(RwLock::new(SharedWebData {
        state : cfg.arbitrary_but_unique_string.clone() ,
        code : String::new(),
    }));
    let ws_handler = std::thread::spawn( || wait_for_response_web_task( data ) );

    webbrowser::open(url.as_str())?;

    let _ = ws_handler.join();

  //let build = client.post(url);

  //let res = apply_timeout_and_send(build, timeout).await?;

  //println!("authorization reply: {} {}", res.status(), res.text().await?);

  Ok(())
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
  -> Result<HomeCoachsData, Error> {

  //let params = [("device_id", "04255185")];
  let build = client.get("https://api.netatmo.com/api/gethomecoachsdata")
    .header("Authorization", String::from("Bearer ") + &token.access_token)
    .header("accept", "application/json");

  let res = apply_timeout_and_send(build, timeout).await?;

   let res = res.json::<HomeCoachsData>().await?;

   Ok( res )
}

