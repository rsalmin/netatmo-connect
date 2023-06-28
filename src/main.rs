use reqwest;
use tokio;
use chrono::naive::NaiveDateTime;
use confy;
use std::time::{Duration, Instant};

use netatmo_connect::*;

#[tokio::main]
async fn main() {
  if let Err( e ) =  main_wrapped().await {
    println!("Error : {:?}", e);
  }
}

async fn main_wrapped() -> Result<(), Error> {

  println!("Configuration path: {:?}", confy::get_configuration_file_path("connect-config", None) );

  let cfg  = confy::load("connect-config", None)?;

  let timeout  = Some( Duration::from_secs(1) );

  let client = reqwest::Client::new();

  let mut token =  get_access_token(&client, &cfg, &timeout).await?;

  let token_duration = token.expires_at - Instant::now();
  println!("Access token expires in {:?}", token_duration);

  loop {

    if token.expires_at < Instant::now() {
      println!("Access token is expired!!!");
      token = get_fresh_token(&client, &cfg, &token, &timeout).await?;
    }

    let res = get_stations_data(&client, &token, &timeout).await?;

     let time_server = NaiveDateTime::from_timestamp_opt(res.time_server, 0);
     match time_server {
       None => println!("Failed to convert server time to NaiveDateTime"),
       Some( v ) => println!("server naive date time: {}", v),
     };

     for d in res.body.devices {
       println!("data from device id : {}", d._id);
       println!("{}", d.dashboard_data);
       for m in d.modules {
         println!("  with module : {}", m._id);
         println!("      Battery : {}%", m.battery_percent);
         println!("      data : {}", m.dashboard_data);
       }
     };

     let hc_data = get_homecoachs_data(&client, &token, &timeout ).await?;

     for d in hc_data.body.devices {
         println!("data from {}", d.station_name);
         println!("{}", d.dashboard_data);
     };


     if token.expires_at < Instant::now() {
       println!("Access token is expired!!!");
     }


     tokio::time::sleep(Duration::from_secs(60)).await;
   };

   //Ok(())
}

