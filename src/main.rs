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

use teslatte::auth::AccessToken;
//use teslatte::vehicles::SetChargeLimit;
//use teslatte::vehicles::Vehicle;
use teslatte::{Api, VehicleId};

use std::{env, fs::create_dir_all, path::PathBuf};

use gettextrs::{bindtextdomain, textdomain};

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

            "47.16610,8.51575".to_string().into()
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

        let api = if access_token_file.exists() {
            let tok = std::fs::read_to_string(&access_token_file).map_err(|e| {
                format!(
                    "Failed to read the tesla access token file {:?}: {}",
                    access_token_file, e
                )
            })?;
            // println!("token: {}", tok);
            Api::new(AccessToken(tok), None)
        } else {
            return Err("not supported yet".to_string());
            //Api::from_interactive_url().await.unwrap()
        };

        Ok(api)
    }

    async fn test_get() -> Result<String, String> {
        let cli = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap();
        let request_builder = cli.get("https://ulrichard.ch/index.html");

        let response_body = request_builder
            .header("Accept", "application/html")
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        //println!("Response: {:?}", response_body);
        Ok(response_body)
    }

    fn get_vehicles(&mut self) -> Result<String, String> {
        //return Ok("Lightning\nHook".to_string());
        let api = self.api.as_ref().ok_or("Not logged in")?;
        let rt = tokio::runtime::Runtime::new().unwrap();

        let index = rt
            .block_on(Greeter::test_get());
            //.map_err(|e| format!("Failed to get index: {}", e))?;
        println!("{:?}", index);

        let vehicles = rt
            .block_on(api.vehicles())
            .map_err(|e| format!("Failed to get vehicles: {}", e))?;
        println!("{:?}", vehicles);
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
