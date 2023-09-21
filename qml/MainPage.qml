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

            Button {
                text: i18n.tr('Login')
                onClicked: {
                    var vehicles = greeter.login().split("\n");
                    vehicle.model = vehicles;
                }
            }

            ComboBox {
                id: vehicle
                onActivated: (i) => {
                    var vehicle_data = greeter.get_vehicle_data(i).split("\n");
                    txt_pos.text = vehicle_data[0];
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

	TextArea {
	    id: eventlog
            Layout.fillWidth: true
            enabled: false
	    text: "node is starting\n\n\n\n\n"
	}

        Timer {
            id: event_timer;
            interval: 2000;
            running: true;
            repeat: true

            onTriggered: {
                eventlog.text = greeter.update_log();
            }
        }

    }
}
