use reqwest;
use serde::Deserialize;
use std::time::{Duration, Instant};
use webbrowser;
use actix_web::{get, web, App, HttpServer};
use actix_web::{HttpResponse, Responder};
use std::sync::{RwLock, Arc};

use super::data::*;
use super::errors::*;

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
