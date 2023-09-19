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

        Button {
            text: i18n.tr('Login')
            onClicked: {
                main_timer.stop();

                var vehicles = greeter.login().split("\n");
                vehicle.model = vehicles;

                main_timer.interval = 1000;
                main_timer.start();
            }
        }

        ComboBox {
            id: vehicle
            onActivated: (i) => {
                var vehicle_data = greeter.get_vehicle_data(i).split("\n");
            }
        }

	TextArea {
	    id: eventlog
            Layout.fillWidth: true
            enabled: false
	    text: "node is starting\n\n\n\n\n"
	}


        Timer {
            id: main_timer;
            interval: 2000;
            running: true;
            repeat: true

            onTriggered: {
                console.time("main timer");
                main_timer.stop();
                eventlog.color = "steelblue"


                eventlog.color = "black"
                main_timer.interval = 20000;
                main_timer.start();
                console.timeEnd("main timer");
            }
        }
        Timer {
            id: event_timer;
            interval: 2000;
            running: true;
            repeat: true

            onTriggered: {
                // console.log("event timer enter");
                event_timer.stop();
                eventlog.color = "steelblue"

                eventlog.text = greeter.update_log();

                eventlog.color = "black"
                event_timer.start();
                // console.log("event timer leave");
            }
        }

    }
}
