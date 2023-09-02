use reqwest;
use serde::Deserialize;
use std::time::{Duration, Instant};
use webbrowser;
use actix_web::{get, web, App, HttpServer};
use actix_web::{HttpResponse, Responder};
use actix_web::dev::ServerHandle;
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
    code: String,
    server_handle: Option<ServerHandle>,
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
    .bind(("127.0.0.1", 8000))?.run();

    let server_handle = server.handle();

    {
        let mut internal_data = data.write()?;

        //set server_handle here, therefore when server will be run it will be already set!
        internal_data.server_handle = Some( server_handle );
    }

    //run server
    let () = server.await?;

    println!("Server finished data.code = {}", data.read()?.code);

    Ok(String::new())
}

async fn wait_for_finish(data : Arc<RwLock<SharedWebData>>) -> Result<(), Error>
{
    loop {
            let mut server_handle : Option<ServerHandle> = None;
            if let Ok( internal_data ) = data.read() {
                if ! internal_data.code.is_empty() {
                    server_handle = Some( internal_data.server_handle.clone().expect("server handle already set") );
                }
            }
            if let Some( handle ) = server_handle {
                handle.stop(true).await; //gracefully
                return Ok(())
            }
        tokio::time::sleep( Duration::from_millis(200) ).await;
    }
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
        server_handle : None,
    }));

    let ws_handle = tokio::task::spawn( wait_for_response_web_task( data.clone() ) );
    let wait_handle = tokio::task::spawn( wait_for_finish( data ) );

    webbrowser::open(url.as_str())?;

    match ws_handle.await {
        Err( e ) => log::warn!("webbrowser task failed with {:?}", e),
        Ok( _ ) => () ,
    };

    match wait_handle.await {
        Err( e ) => log::warn!("wait  task failed with {:?}", e),
        Ok( o ) => o? ,
    };

  //let build = client.post(url);

  //let res = apply_timeout_and_send(build, timeout).await?;

  //println!("authorization reply: {} {}", res.status(), res.text().await?);

  Ok(())
}
