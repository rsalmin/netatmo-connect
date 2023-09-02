use serde::{Deserialize, Serialize};
use chrono::naive::NaiveDateTime;
use std::fmt;

#[derive(Deserialize, Debug)]
pub struct StationsData {
  pub body : StationsDataBody,
  pub time_server : i64,
}

#[derive(Deserialize, Debug)]
pub struct StationsDataBody {
  pub devices : Vec<StationsDataDevice>,
}

#[derive(Deserialize, Debug)]
pub struct StationsDataDevice {
  pub _id : String,
  pub dashboard_data : StationsDataDeviceData,
  pub modules : Vec<StationsDataModule>,
}

#[derive(Deserialize, Debug)]
pub struct StationsDataModule {
  pub _id : String,
  pub battery_percent : i32,
  pub dashboard_data : StationsDataModuleData,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct StationsDataModuleData {
  pub time_utc: i64,
  pub Temperature : f32,
  pub Humidity : i32,
  pub temp_trend : String,
}

impl fmt::Display for StationsDataModuleData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       let time = NaiveDateTime::from_timestamp_opt(self.time_utc, 0);
       match time {
         None => write!(f, "Failed to convert time from {}", self.time_utc)?,
         Some( v ) => write!(f, "Time: {}", v)?,
        };
        write!(f, "  Temperature: {} ({})", self.Temperature, self.temp_trend)?;
        write!(f, "  Humidity: {}", self.Humidity)?;
        Ok(())
    }
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct StationsDataDeviceData {
  pub time_utc: i64,
  pub Temperature : f32,
  pub CO2 : i32,
  pub Humidity : i32,
  pub Noise : i32,
  pub Pressure: f32,
  pub AbsolutePressure : f32,
  pub temp_trend : String,
  pub pressure_trend : String,
}

impl fmt::Display for StationsDataDeviceData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       let time = NaiveDateTime::from_timestamp_opt(self.time_utc, 0);
       match time {
         None => write!(f, "Failed to convert time from {}", self.time_utc)?,
         Some( v ) => write!(f, "Time: {}", v)?,
        };
        write!(f, "  Temperature: {} ({})", self.Temperature, self.temp_trend)?;
        write!(f, "  CO2: {}", self.CO2)?;
        write!(f, "  Humidity: {}%", self.Humidity)?;
        write!(f, "  Noise: {}db", self.Noise)?;
        write!(f, "  Pressure: {}mmHg ({})", self.Pressure, self.pressure_trend)?;
        write!(f, "  AbsolutePressure: {}", self.AbsolutePressure)?;
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
pub struct HomeCoachsData {
  pub body : HomeCoachsDataBody,
  pub time_server : i64,
}

#[derive(Deserialize, Debug)]
pub struct HomeCoachsDataBody {
  pub devices : Vec<HomeCoachsDataDevice>,
}

#[derive(Deserialize, Debug)]
pub struct HomeCoachsDataDevice {
  pub _id : String,
  pub station_name : String,
  pub dashboard_data : HomeCoachsDeviceData,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct HomeCoachsDeviceData {
  pub time_utc: i64,
  pub Temperature : f32,
  pub CO2 : i32,
  pub Humidity : i32,
  pub Noise : i32,
  pub Pressure: f32,
  pub AbsolutePressure : f32,
  pub health_idx : i32,
}

impl fmt::Display for HomeCoachsDeviceData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       let time = NaiveDateTime::from_timestamp_opt(self.time_utc, 0);
       match time {
         None => write!(f, "Failed to convert time from {}", self.time_utc)?,
         Some( v ) => write!(f, "Time: {}", v)?,
        };
        write!(f, "  Temperature: {}", self.Temperature)?;
        write!(f, "  CO2: {}", self.CO2)?;
        write!(f, "  Humidity: {}%", self.Humidity)?;
        write!(f, "  Noise: {}db", self.Noise)?;
        write!(f, "  Pressure: {}mmHg", self.Pressure)?;
        write!(f, "  AbsolutePressure: {}", self.AbsolutePressure)?;
        write!(f, "  health_idx: {}", self.health_idx)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConnectConfig {
    pub client_id: String,
    pub client_secret: String,
    pub username: String,
    pub password: String,
    pub arbitrary_but_unique_string: String,
}

/// Default for initial save ;)
impl ::std::default::Default for ConnectConfig  {
    fn default() -> Self {
     Self { client_id: "cliend_id".into(), client_secret : "client_secret".into(),
                username : "username".into(), password : "password".into(),
                arbitrary_but_unique_string : "arbitrary_not_so_unique_string".into(), }
    }
}

