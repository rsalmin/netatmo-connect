use reqwest;
use tokio;
use chrono::naive::NaiveDateTime;
use confy;
use std::time::{Duration, Instant};
use env_logger;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use netatmo_connect::*;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    let stop_flag = Arc::new(AtomicBool::new(false));

    let stop_flag_clone = stop_flag.clone();
    ctrlc::set_handler( move || {
        log::info!("received Ctrl+C! finishing...");
        stop_flag_clone.store(true, Ordering::Relaxed);
      })
      .expect("Error setting Ctrl-C handler");

    if let Err( e ) =  main_wrapped(stop_flag).await {
        log::error!("Exit: {:?}", e);
    }
}

async fn main_wrapped(stop_flag : Arc<AtomicBool>) -> Result<(), Error> {

  log::info!("Configuration path: {:?}", confy::get_configuration_file_path("connect-config", None) );

  let cfg  = confy::load("connect-config", None)?;

  let timeout  = Some( Duration::from_secs(1) );

  let client = reqwest::Client::new();

  let mut token =  authorize(&client, &cfg, &Option::None).await?;


  //get_client_access_token(&client, &cfg, &timeout).await?;

  let token_duration = token.expires_at - Instant::now();
  log::info!("Access token expires in {:?}", token_duration);

  'main_loop: loop {

    if token.expires_at < Instant::now() {
      println!("Access token is expired!!!");
      token = get_fresh_token(&client, &cfg, &token, &timeout).await?;
    }

    let res = get_stations_data(&client, &token, &timeout).await?;

     let time_server = NaiveDateTime::from_timestamp_opt(res.time_server, 0);
     match time_server {
       None => log::error!("Failed to convert server time to NaiveDateTime"),
       Some( v ) => log::info!("server naive date time: {}", v),
     };

     for d in res.body.devices {
       println!("data from device id : {}", d._id);
       println!("    {}", d.dashboard_data);
       for m in d.modules {
         println!("    with module : {}", m._id);
         println!("        Battery : {}%", m.battery_percent);
         println!("        data : {}", m.dashboard_data);
       }
     };

     match get_homecoachs_data(&client, &token, &timeout ).await {
         Err( e ) => log::error!("Failed to get homecoachs data : {:?}", e),
         Ok( data ) => {
             for d in data.body.devices {
                 println!("data from {}", d.station_name);
                 if let Some( dashboard_data ) = d.dashboard_data {
                     println!("    {}", dashboard_data);
                 }
                 else {
                     println!("    Missing....");
                 }
             };
         },
     };


     if token.expires_at < Instant::now() {
       println!("Access token is expired!!!");
     }

    for _ in 0..60 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        if stop_flag.load(Ordering::Relaxed) {
            break 'main_loop;
        }
    };
  };

    log::info!("finished succesefully");
    Ok(())
}

