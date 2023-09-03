use reqwest;
use serde::Deserialize;
use std::time::Duration;
use webbrowser;
use actix_web::{get, web, App, HttpServer};
use actix_web::{HttpResponse, Responder};
use actix_web::dev::ServerHandle;
use std::sync::{RwLock, Arc};

use super::data::*;
use super::errors::*;
use super::funs::{apply_timeout_and_send, convert_token, AccessToken, AccessTokenJSON};

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

    log::debug!("Authorization response handler");

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

async fn wait_for_response_web_task(data : Arc<RwLock<SharedWebData>>) -> Result<String, Error>
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

    Ok( data.read()?.code.clone() )
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

async fn get_code_string(cfg : &ConnectConfig) -> Result<String, Error>
{
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

    wait_handle.await??;
    log::debug!("wait process finished");
    let r = ws_handle.await??;
    log::debug!("web server process finished");
    Ok(r)
}

pub async fn authorize(client : &reqwest::Client, cfg : &ConnectConfig, timeout : &Option<Duration>) -> Result<AccessToken, Error>
{
    let code_string = get_code_string(cfg).await?;

    log::info!("got code string {}", code_string);

    let params = [("grant_type", "authorization_code"),
                            ("client_id", &cfg.client_id),
                            ("client_secret", &cfg.client_secret),
                            ("code", &code_string),
                            ("redirect_uri", &String::from("http://localhost:8000/api/authorization_response")),
                            ("scope", "read_station read_homecoach")];

  let build = client.post("https://api.netatmo.com/oauth2/token").form(&params);

  let res = apply_timeout_and_send(build, timeout).await?;

  let res = res.json::<AccessTokenJSON>().await?;

  convert_token(res)
}
