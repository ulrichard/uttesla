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
use teslatte::Api;

use std::{env, fs::create_dir_all, path::PathBuf};

use gettextrs::{bindtextdomain, textdomain};

#[derive(QObject, Default)]
struct Greeter {
    base: qt_base_class!(trait QObject),
    eventlog: std::collections::VecDeque<String>,
    api: Option<Api>,

    login: qt_method!(
        fn login(&mut self) -> QString {
            self.api = self.log_err(self.log_in());
            self.log_err_or(self.get_vehicles(), "".to_string()).into()
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
            Api::new(AccessToken(tok), None)
        } else {
            return Err("not supported yet".to_string());
            //Api::from_interactive_url().await.unwrap()
        };

        Ok(api)
    }

    fn get_vehicles(&self) -> Result<String, String> {
        let api = self.api.ok_or("Not logged in")?;
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        let vehicles = rt.block_on(api.vehicles());

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
    #[should_panic(expected = "Unknown input format")]
    fn test_login_fail() {
        let api = match env::var("TESLA_ACCESS_TOKEN") {
            Ok(t) => Api::new(AccessToken(t), None),
            Err(_) => Api::from_interactive_url().await.unwrap(),
        };

        let vehicles = api.vehicles().await.unwrap();
        dbg!(&vehicles);
    }
}
