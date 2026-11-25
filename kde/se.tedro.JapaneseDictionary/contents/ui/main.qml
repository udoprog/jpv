pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts

import org.kde.kirigami as Kirigami
import org.kde.plasma.components as PlasmaComponents3
import org.kde.plasma.extras as PlasmaExtras
import org.kde.plasma.plasmoid
import org.kde.plasma.workspace.dbus as DBus

PlasmoidItem {
    id: root

    readonly property string service: "se.tedro.JapaneseDictionary"
    readonly property string objectPath: "/se/tedro/JapaneseDictionary"

    // Whether the service is capturing the clipboard, as last read back.
    property bool capturing: false
    property string errorText: ""

    Plasmoid.icon: "se.tedro.JapaneseDictionary"
    toolTipMainText: "Japanese Dictionary"
    toolTipSubText: capturing ? "Capturing the clipboard" : ""

    function call(member, args, signature, resolve, reject) {
        const message = {
            service: root.service,
            path: root.objectPath,
            iface: root.service,
            member: member,
            arguments: args,
        };

        if (signature) {
            message.signature = signature;
        }

        // Both callbacks are given the finished DBusPendingReply.
        DBus.SessionBus.asyncCall(message, reply => resolve(reply.value), reply => {
            const error = reply.error;
            root.errorText = error.message || error.name || "D-Bus call failed";

            if (reject) {
                reject();
            }
        });
    }

    function refresh() {
        call("GetCaptureClipboard", [], "", capture => {
            root.errorText = "";
            root.capturing = !!capture;
        });
    }

    function setCapture(enabled) {
        // Read the state back either way, so the switch shows what the
        // service actually does.
        call("SetCaptureClipboard", [new DBus.bool(enabled)], "(b)", () => refresh(), () => refresh());
    }

    function openDictionary() {
        call("GetPort", [], "", port => {
            root.errorText = "";
            Qt.openUrlExternally("http://localhost:" + port);
            root.expanded = false;
        });
    }

    onExpandedChanged: {
        if (expanded) {
            refresh();
        }
    }

    // Only reads the state when the service is up, so that loading the
    // widget does not start it through D-Bus activation.
    DBus.DBusServiceWatcher {
        id: watcher
        busType: DBus.BusType.Session
        watchedService: root.service
        onRegisteredChanged: {
            if (registered) {
                root.refresh();
            } else {
                root.capturing = false;
            }
        }
    }

    Component.onCompleted: {
        if (watcher.registered) {
            refresh();
        }
    }

    compactRepresentation: MouseArea {
        id: compact

        property bool wasExpanded: false

        Layout.minimumWidth: Kirigami.Units.iconSizes.small
        Layout.minimumHeight: Kirigami.Units.iconSizes.small

        acceptedButtons: Qt.LeftButton
        hoverEnabled: true

        onPressed: compact.wasExpanded = root.expanded
        onClicked: root.expanded = !compact.wasExpanded

        // The capturing state, like the GNOME extension's `:capture` style.
        Rectangle {
            anchors.fill: parent
            radius: Kirigami.Units.cornerRadius
            color: Qt.rgba(146 / 255, 45 / 255, 45 / 255, 1)
            visible: root.capturing
        }

        Kirigami.Icon {
            anchors.fill: parent
            anchors.margins: root.capturing ? Kirigami.Units.smallSpacing / 2 : 0
            source: Plasmoid.icon
            active: compact.containsMouse
        }
    }

    fullRepresentation: PlasmaExtras.Representation {
        Layout.minimumWidth: Kirigami.Units.gridUnit * 14
        Layout.preferredHeight: column.implicitHeight + Kirigami.Units.largeSpacing * 2

        ColumnLayout {
            id: column

            anchors.fill: parent
            anchors.margins: Kirigami.Units.largeSpacing
            spacing: Kirigami.Units.smallSpacing

            PlasmaComponents3.ToolButton {
                Layout.fillWidth: true
                text: "Open dictionary"
                icon.name: "internet-web-browser-symbolic"
                onClicked: root.openDictionary()
            }

            PlasmaComponents3.Switch {
                id: captureSwitch

                Layout.fillWidth: true
                text: "Capture clipboard 📋"
                checked: root.capturing
                onToggled: root.setCapture(captureSwitch.checked)
            }

            PlasmaComponents3.Label {
                Layout.fillWidth: true
                visible: root.errorText !== ""
                text: root.errorText
                wrapMode: Text.Wrap
                color: Kirigami.Theme.negativeTextColor
            }
        }
    }
}
