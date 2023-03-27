use serde::Deserialize;
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

