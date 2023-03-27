use reqwest;
use tokio;
use chrono::naive::NaiveDateTime;
use confy;

mod netatmo;
use netatmo::*;

#[tokio::main]
async fn main() {
  if let Err( e ) =  main_wrapped().await {
    println!("Error : {:?}", e);
  }
}

async fn main_wrapped() -> Result<(), Error> {

  println!("Configuration path: {:?}", confy::get_configuration_file_path("connect-config", None) );

  let cfg  = confy::load("connect-config", None)?;

  let client = reqwest::Client::new();

  let token =  get_access_token(&client, &cfg).await?;

  let res = get_stations_data(&client, &token).await?;

   let time_server = NaiveDateTime::from_timestamp_opt(res.time_server, 0);
   match time_server {
     None => println!("Failed to convert server time to NaiveDateTime"),
     Some( v ) => println!("server naive date time: {}", v),
   };

   for d in res.body.devices {
     println!("Device id : {}", d._id);
     println!("data : {}", d.dashboard_data);
     for m in d.modules {
       println!("  Module : {}", m._id);
       println!("  Battery : {}%", m.battery_percent);
       println!("  data : {}", m.dashboard_data);
     }
   };
   Ok(())
}

