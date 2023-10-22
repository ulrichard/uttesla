/*
 * Copyright (C) 2022  Richard Ulrich
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 3.
 *
 * uttesla is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

#[macro_use]
extern crate cstr;
extern crate cpp;
#[macro_use]
extern crate qmetaobject;
use qmetaobject::*;
use qt_core::{q_standard_paths::StandardLocation, QStandardPaths};

mod constants;
mod qrc;

use teslatte::auth::{AccessToken, RefreshToken};
use serde::Serialize;
use teslatte::{vehicles::{SetTemperatures, SetChargeLimit}, Api, VehicleId};

use std::{env, fs::create_dir_all, path::PathBuf};

use gettextrs::{bindtextdomain, textdomain};

#[derive(Debug, Clone, Serialize)]
struct ReducedVehicleData {
    pub gps_pos: String,
    pub inside_temp: String,
    pub outside_temp: String,
    pub driver_temp_setting: i64,
    pub hvac_enabled: bool,
    pub battery_level: i64,
    pub battery_range: f64,
    pub charge_rate: f64,
    pub charge_energy_added: f64,
    pub charge_limit: i64,
}

#[derive(QObject, Default)]
struct Greeter {
    base: qt_base_class!(trait QObject),
    eventlog: std::collections::VecDeque<String>,
    api: Option<Api>,
    vehicles: Vec<(VehicleId, String)>,

    login: qt_method!(
        fn login(&mut self) -> QString {
            self.api = self.log_err(self.log_in());
            let names = self.get_vehicles();
            self.log_err_or(names, "".to_string()).into()
        }
    ),
    update_log: qt_method!(
        fn update_log(&mut self) -> QString {
            self.eventlog.truncate(5);
            self.eventlog
                .iter()
                .fold("".to_string(), |acc, msg| format!("{}\n{}", acc, msg))
                .trim()
                .into()
        }
    ),
    get_vehicle_data: qt_method!(
        fn get_vehicle_data(&mut self, idx: i64) -> QString {
            let vehicle = self.get_vehicle(idx);
            println!("{}", vehicle.clone().unwrap_or_else(|e| e.to_string()));
            self.log_err_or(vehicle, "".to_string()).into()
        }
    ),
    hvac: qt_method!(
        fn hvac(&mut self, idx: i64, enable: bool, temp: i64) {
            let res = self.enable_hvac(idx, enable, temp);
            let _ = self.log_err(res);
        }
    ),
    doors: qt_method!(
        fn doors(&mut self, idx: i64, do_open: bool) {
            let res = self.lock_doors(idx, do_open);
            let _ = self.log_err(res);
        }
    ),
    charge: qt_method!(
        fn charge(&mut self, idx: i64, do_start: bool, charge_limit: u8) {
            let res = self.charging(idx, do_start, charge_limit);
            let _ = self.log_err(res);
        }
    ),
    honk: qt_method!(
        fn honk(&mut self, idx: i64) {
            let res = self.honk_horn(idx);
            let _ = self.log_err(res);
        }
    ),
    flash: qt_method!(
        fn flash(&mut self, idx: i64) {
            let res = self.flash_lights(idx);
            let _ = self.log_err(res);
        }
    ),
    drive: qt_method!(
        fn drive(&mut self, idx: i64) {
            let res = self.remote_start_drive(idx);
            let _ = self.log_err(res);
        }
    ),
}

impl Greeter {
    fn log_in(&self) -> Result<Api, String> {
        let app_data_path =
            unsafe { QStandardPaths::writable_location(StandardLocation::AppDataLocation) };
        let app_data_path = PathBuf::from(app_data_path.to_std_string());
        create_dir_all(&app_data_path).unwrap();
        let access_token_file = app_data_path.join("tesla_access_token.txt");
        let refresh_token_file = app_data_path.join("tesla_refresh_token.txt");

        let api = if refresh_token_file.exists() {
            let tok = std::fs::read_to_string(&refresh_token_file)
                .map_err(|e| {
                    format!(
                        "Failed to read the tesla refresh token file {:?}: {}",
                        refresh_token_file, e
                    )
                })?
                .trim()
                .to_string();

            let rt = tokio::runtime::Runtime::new().unwrap();

            let api = rt
                .block_on(Api::from_refresh_token(&RefreshToken(tok)))
                .map_err(|e| format!("failed to refresh token: {:?}", e))?;
            std::fs::write(&access_token_file, &api.access_token.0)
                .map_err(|e| format!("failed to write access_token: {:?}", e))?;
            if let Some(refresh_token) = &api.refresh_token {
                std::fs::write(&refresh_token_file, &refresh_token.0)
                    .map_err(|e| format!("failed to write refresh_token: {:?}", e))?;
            }
            api
        } else if access_token_file.exists() {
            let tok = std::fs::read_to_string(&access_token_file)
                .map_err(|e| {
                    format!(
                        "Failed to read the tesla access token file {:?}: {}",
                        access_token_file, e
                    )
                })?
                .trim()
                .to_string();
            // println!("token: {}", tok);
            Api::new(AccessToken(tok), None)
        } else {
            return Err("not supported yet".to_string());
            //Api::from_interactive_url().await.unwrap()
        };

        Ok(api)
    }

    fn get_vehicles(&mut self) -> Result<String, String> {
        let api = self.api.as_ref().ok_or("Not logged in")?;
        let rt = tokio::runtime::Runtime::new().unwrap();

        let vehicles = rt
            .block_on(api.vehicles())
            .map_err(|e| format!("Failed to get vehicles: {}", e))?;
        self.vehicles = vehicles
            .data()
            .iter()
            .map(|v| (v.id.clone(), v.display_name.clone()))
            .collect();
        Ok(self
            .vehicles
            .iter()
            .fold("".to_string(), |acc, (_id, name)| {
                format!("{}\n{}", acc, name)
            })
            .trim()
            .to_string())
    }

    fn get_vehicle(&mut self, idx: i64) -> Result<String, String> {
        let api = self.api.as_ref().ok_or("Not logged in")?;
        let rt = tokio::runtime::Runtime::new().unwrap();

        let vid = &self.vehicles[idx as usize].0;
        let _ = rt
            .block_on(api.wake_up(vid))
            .map_err(|e| format!("Failed to wake up vehicle {}: {}", idx, e))?;
        let vehicle = rt
            .block_on(api.vehicle_data(vid))
            .map_err(|e| format!("Failed to get vehicle {}: {}", idx, e))?;

        let gps_pos = if let Some(drive_state) = &vehicle.drive_state {
            format!("{},{}", drive_state.latitude, drive_state.longitude)
        } else {
            "".to_string()
        };
        let inside_temp = if let Some(climate_state) = &vehicle.climate_state {
            if let Some(itemp) = climate_state.inside_temp {
                format!("{}", itemp)
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };
        let outside_temp = if let Some(climate_state) = &vehicle.climate_state {
            if let Some(otemp) = climate_state.outside_temp {
                format!("{}", otemp)
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };
        let driver_temp_setting = if let Some(climate_state) = &vehicle.climate_state {
            climate_state.driver_temp_setting as i64
        } else {
            20
        };
        let hvac_enabled = if let Some(climate_state) = &vehicle.climate_state {
            climate_state.fan_status != 0
        } else {
            false
        };
        let (battery_level, battery_range, charge_rate, charge_energy_added, charge_limit) =
            if let Some(charge_state) = &vehicle.charge_state {
                (
                    charge_state.battery_level,
                    charge_state.battery_range * 1.609344,
                    charge_state.charge_rate,
                    charge_state.charge_energy_added,
                    charge_state.charge_limit_soc,
                )
            } else {
                (0, 0.0, 0.0, 0.0, 80)
            };
        let vehicle_data = ReducedVehicleData {
            gps_pos,
            inside_temp,
            outside_temp,
            driver_temp_setting,
            hvac_enabled,
            battery_level,
            battery_range,
            charge_rate,
            charge_energy_added,
            charge_limit,
        };
        serde_json::to_string(&vehicle_data)
            .map_err(|e| format!("Failed to serialize ReducedVehicleData: {:?}", e))
    }

    fn enable_hvac(&mut self, idx: i64, enable: bool, temp: i64) -> Result<(), String> {
        let api = self.api.as_ref().ok_or("Not logged in")?;
        let rt = tokio::runtime::Runtime::new().unwrap();
        let vid = &self.vehicles[idx as usize].0;

        let temps = SetTemperatures {
            driver_temp: temp as f32,
            passenger_temp: temp as f32,
        };
        let _ = rt
            .block_on(api.set_temps(vid, &temps))
            .map_err(|e| format!("Failed to set hvac temperature {}: {}", idx, e))?;
        let _ = if enable {
            rt.block_on(api.auto_conditioning_start(vid))
        } else {
            rt.block_on(api.auto_conditioning_stop(vid))
        }
        .map_err(|e| format!("Failed to enable or disable hvac {}: {}", idx, e))?;

        Ok(())
    }

    fn lock_doors(&mut self, idx: i64, do_open: bool) -> Result<(), String> {
        let api = self.api.as_ref().ok_or("Not logged in")?;
        let rt = tokio::runtime::Runtime::new().unwrap();
        let vid = &self.vehicles[idx as usize].0;

        let _ = if do_open {
            rt.block_on(api.door_unlock(vid))
        } else {
            rt.block_on(api.door_lock(vid))
        }
        .map_err(|e| format!("Failed to (un)-lock the doors {}: {}", idx, e))?;

        Ok(())
    }

    fn charging(&mut self, idx: i64, do_start: bool, charge_limit: u8) -> Result<(), String> {
        let api = self.api.as_ref().ok_or("Not logged in")?;
        let rt = tokio::runtime::Runtime::new().unwrap();
        let vid = &self.vehicles[idx as usize].0;

        let _ = if do_start {
            let limit = SetChargeLimit{percent: charge_limit};
            rt.block_on(api.set_charge_limit(vid, &limit))
                .map_err(|e| format!("Failed to set charge limit {}: {}", idx, e))?;
            rt.block_on(api.charge_start(vid))
        } else {
            rt.block_on(api.charge_stop(vid))
        }
        .map_err(|e| format!("Failed to start/stop charging {}: {}", idx, e))?;

        Ok(())
    }

    fn honk_horn(&mut self, idx: i64) -> Result<(), String> {
        let api = self.api.as_ref().ok_or("Not logged in")?;
        let rt = tokio::runtime::Runtime::new().unwrap();
        let vid = &self.vehicles[idx as usize].0;

        let _ = rt
            .block_on(api.honk_horn(vid))
            .map_err(|e| format!("Failed to honk the horn {}: {}", idx, e))?;

        Ok(())
    }

    fn flash_lights(&mut self, idx: i64) -> Result<(), String> {
        let api = self.api.as_ref().ok_or("Not logged in")?;
        let rt = tokio::runtime::Runtime::new().unwrap();
        let vid = &self.vehicles[idx as usize].0;

        let _ = rt
            .block_on(api.flash_lights(vid))
            .map_err(|e| format!("Failed to flash the lights {}: {}", idx, e))?;

        Ok(())
    }

    fn remote_start_drive(&mut self, idx: i64) -> Result<(), String> {
        let api = self.api.as_ref().ok_or("Not logged in")?;
        let rt = tokio::runtime::Runtime::new().unwrap();
        let vid = &self.vehicles[idx as usize].0;

        let _ = rt
            .block_on(api.remote_start_drive(vid))
            .map_err(|e| format!("Failed allow keyless driving {}: {}", idx, e))?;

        Ok(())
    }

    fn log_err<T>(&mut self, res: Result<T, String>) -> Option<T> {
        match res {
            Ok(d) => Some(d),
            Err(err) => {
                eprintln!("{}", err);
                self.eventlog.push_front(err.clone());
                //panic!("{}", err);
                None
            }
        }
    }

    fn log_err_or<T>(&mut self, res: Result<T, String>, fallback: T) -> T {
        match res {
            Ok(d) => d,
            Err(err) => {
                eprintln!("{}", err);
                self.eventlog.push_front(err);
                fallback
            }
        }
    }
}

#[cfg(not(test))]
fn main() {
    init_gettext();
    unsafe {
        cpp! { {
            #include <QtCore/QCoreApplication>
            #include <QtCore/QString>
        }}
        cpp! {[]{
            QCoreApplication::setApplicationName(QStringLiteral("uttesla.ulrichard"));
        }}
    }
    QQuickStyle::set_style("Suru");
    qrc::load();
    qml_register_type::<Greeter>(cstr!("Greeter"), 1, 0, cstr!("Greeter"));
    let mut engine = QmlEngine::new();

    println!("Loading file /qml/uttesla.qml.");
    engine.load_file("qrc:/qml/uttesla.qml".into());
    println!("Entering the QML main loop.");
    engine.exec();
}

#[cfg(not(test))]
fn init_gettext() {
    let domain = "uttesla.ulrichard";
    textdomain(domain).expect("Failed to set gettext domain");

    let app_dir = env::var("APP_DIR").expect("Failed to read the APP_DIR environment variable");

    let mut app_dir_path = PathBuf::from(app_dir);
    if !app_dir_path.is_absolute() {
        app_dir_path = PathBuf::from("/usr");
    }

    let path = app_dir_path.join("share/locale");

    bindtextdomain(domain, path.to_str().unwrap()).expect("Failed to bind gettext domain");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login() {
        let app_data_path =
            unsafe { QStandardPaths::writable_location(StandardLocation::AppDataLocation) };
        let app_data_path = PathBuf::from(app_data_path.to_std_string());
        create_dir_all(&app_data_path).unwrap();
        let access_token_file = app_data_path.join("tesla_access_token.txt");

        println!("access_token_file: {}", access_token_file.display());
        let tok = std::fs::read_to_string(&access_token_file).unwrap();
        println!("access_token: {}", tok);
        let api = Api::new(AccessToken(tok), None);

        let mut rt = tokio::runtime::Runtime::new().unwrap();
        let vehicles = rt.block_on(api.vehicles()).unwrap();
        println!("{:?}", vehicles);
        dbg!(&vehicles);
    }

    #[test]
    #[should_panic(expected = "DecodeJsonError")]
    fn test_login_fail() {
        let tok = "ThisIsNotAValidAccessToken".to_string();
        println!("access_token: {}", tok);
        let api = Api::new(AccessToken(tok), None);

        let mut rt = tokio::runtime::Runtime::new().unwrap();
        let vehicles = rt.block_on(api.vehicles()).unwrap();
        println!("{:?}", vehicles);
        dbg!(&vehicles);
    }
}
