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

import QtQuick 2.7
import QtQuick.Controls 2.2
import Ubuntu.Components 1.3
import QtQuick.Layouts 1.3
import Qt.labs.settings 1.0
import QtQml 2.12

import Greeter 1.0

// for widgets visit:
// https://doc.qt.io/qt-6/qtquick-controls2-qmlmodule.html

Page {
    id: mainPage

    Greeter {
        id: greeter
    }

    anchors.fill: parent

    header: PageHeader {
        id: header
        title: i18n.tr('uttesla')
    }

    ColumnLayout {
        spacing: units.gu(2)
        anchors {
            margins: units.gu(2)
            top: header.bottom
            left: parent.left
            right: parent.right
            bottom: parent.bottom
        }

        // vehicles
        RowLayout {
            spacing: units.gui(2)

            /*
            Button {
                text: i18n.tr('Login')
                onClicked: {
                    var vehicles = greeter.login().split("\n");
                    vehicle.model = vehicles;
                }
            }
            */

            Label {
                id: lbl_vehicle
                text: i18n.tr('Vehicle')
            }

            ComboBox {
                id: vehicle
                onActivated: (i) => {
                    var vehicle_data = JSON.parse(greeter.get_vehicle_data(i));
                    txt_pos.text = vehicle_data.gps_pos;
                    lbl_temp.text = "Temperature Out:" + vehicle_data.outside_temp + " In: " + vehicle_data.inside_temp;
		    spb_temp.value = vehicle_data.driver_temp_setting;
		    chk_hvac.checked = vehicle_data.hvac_enabled;
		    lbl_batt.text = "Battery: " + vehicle_data.battery_level + "%  " + vehicle_data.battery_range.toFixed(1) + "km  " + vehicle_data.charge_rate + "chg rate  " + vehicle_data.charge_energy_added.toFixed(1) + "kWh";
		    spb_chg_limit.value = vehicle_data.charge_limit;
                }
            }
        }

	// position
        RowLayout {
            spacing: units.gui(2)

            Label {
                id: lbl_pos
                text: i18n.tr('Pos')
            }

            TextField {
                id: txt_pos
                placeholderText: i18n.tr('Position')
                enabled: false
                implicitWidth: 150
            }

            Button {
                id: btn_pos
                text: i18n.tr('Map')
                onClicked: {
                    var url = 'geo:' + txt_pos.text;
                    Qt.openUrlExternally(url);
                }
            }
        }

	// temperature
        RowLayout {
            spacing: units.gui(2)

            Label {
                id: lbl_temp
                text: i18n.tr('Temperature')
            }
	}
        RowLayout {
            spacing: units.gui(2)

            CheckBox {
                id: chk_hvac
                text: i18n.tr('HVAC')
                onClicked: {
                    greeter.hvac(vehicle.currentIndex, chk_hvac.checked, spb_temp.value);
                }
            }

            SpinBox {
                id: spb_temp
                value: 20
                from: 10
                to: 30
                implicitWidth: 70
            }
        }

	// battery
        RowLayout {
            spacing: units.gui(2)

            Label {
                id: lbl_batt
                text: i18n.tr('Battery')
            }
	}
        RowLayout {
            spacing: units.gui(2)

            Button {
                id: btn_charge_start
                text: i18n.tr('Start charging')
                onClicked: {
                    greeter.charge(vehicle.currentIndex, true, spb_chg_limit.value);
                }
            }

            Button {
                id: btn_charge_stop
                text: i18n.tr('Stop charging')
                onClicked: {
                    greeter.charge(vehicle.currentIndex, false, spb_chg_limit.value);
                }
            }

            Label {
                id: lbl_chg_limit
                text: i18n.tr('to %')
            }
            SpinBox {
                id: spb_chg_limit
                value: 80
                from: 50
                to: 100
                stepSize: 5
                implicitWidth: 80
            }
        }

	// alerts
        RowLayout {
            spacing: units.gui(2)

            Button {
                id: btn_honk
                text: i18n.tr('Honk')
                onClicked: {
                    greeter.honk(vehicle.currentIndex);
                }
            }

            Button {
                id: btn_flash
                text: i18n.tr('Flash')
                onClicked: {
                    greeter.flash(vehicle.currentIndex);
                }
            }
        }

	// doors
        RowLayout {
            spacing: units.gui(2)

            Button {
                id: btn_unlock
                text: i18n.tr('Unlock')
                onClicked: {
                    greeter.doors(vehicle.currentIndex, true);
                }
            }

            Button {
                id: btn_lock
                text: i18n.tr('Lock')
                onClicked: {
                    greeter.doors(vehicle.currentIndex, false);
                }
            }

            Button {
                id: btn_drive
                text: i18n.tr('Drive')
                onClicked: {
                    greeter.drive(vehicle.currentIndex);
                }
            }
        }

	TextArea {
	    id: eventlog
            Layout.fillWidth: true
            enabled: false
	}

        Timer {
            id: log_timer;
            interval: 2000;
            running: true;
            repeat: true

            onTriggered: {
                eventlog.text = greeter.update_log();
            }
        }

        Timer {
            id: refresh_timer;
            interval: 10000;
            running: true;
            repeat: true

            onTriggered: {
                vehicle.activated(vehicle.currentIndex);
            }
        }

    }

    Component.onCompleted: {
        var vehicles = greeter.login().split("\n");
        vehicle.model = vehicles;
	vehicle.activated(0);
    }
}
