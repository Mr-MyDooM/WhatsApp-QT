import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "."                    // local components
import "../styles"            // stylesheet
Rectangle {
    id: sidebar
    // currentTheme: mainWindow.currentTheme
    property string currentTheme
    property var appController
    property var stackLayout
    property var tabsModel
    property int currentIndex: stackLayout ? stackLayout.currentIndex : 0
    property real collapsedWidth: 64
    property real expandedWidth: 200
    readonly property int maxTabNameLength: 40

    // Trim, clamp to a sidebar-sized label, and fall back if nothing survives.
    // Exposed so the rename UI can reuse it once main.qml implements renaming.
    function sanitizeTabName(name, fallback) {
        var clean = (name === undefined || name === null) ? "" : String(name).trim();
        if (clean.length === 0)
            return fallback;
        return clean.length > maxTabNameLength ? clean.substring(0, maxTabNameLength) : clean;
    }

    // "WhatsApp #" + (count + 1) collides after a tab is removed (remove 1 of 3
    // and the next new tab is named "WhatsApp #3" again). Scan for a free number.
    function nextTabName() {
        var taken = {};
        for (var i = 0; i < tabsModel.count; i++) {
            var entry = tabsModel.get(i);
            if (entry && entry.name)
                taken[entry.name] = true;
        }
        for (var n = 1; n <= tabsModel.count + 1; n++) {
            var candidate = "WhatsApp #" + n;
            if (!taken[candidate])
                return candidate;
        }
        return "WhatsApp #" + (tabsModel.count + 1);
    }
    // settingsDialog is scoped to this file; exposed so Ctrl+, in main.qml can reach it.
    function openSettings() {
        settingsDialog.open();
    }
    Layout.preferredWidth: sidebarVisible ? expandedWidth : collapsedWidth
    Behavior on Layout.preferredWidth {
                    NumberAnimation {
                        duration: 150
                        easing.type: Easing.InOutQuad
                    }
                }
    // Layout.preferredWidth: 64
    Layout.fillHeight: true
    color: currentTheme === "dark" ? "#232323" : "#ececec"
    border.color: currentTheme === "dark" ? "#444" : "#bbb"
    border.width: 0.75

    ColumnLayout {
        id: mainLayout
        anchors.fill: parent
        spacing: 0
        // --- Tab icons (accounts) and "+" button ---
        Column {
            id: tabCol
            Layout.alignment: Qt.AlignHCenter
            spacing: 6
            // Tab icons
            Repeater {
                model: tabsModel
                Rectangle {
                    id: tabIconRect
                    width: 48; height: 48
                    radius: 24
                    color: currentIndex === index
                        ? (currentTheme === "dark" ? "#00bfa5" : "#42a5f5")
                        : "transparent"
                    border.width: currentIndex === index ? 0 : 1
                    border.color: currentTheme === "dark" ? "#444" : "#bbb"
                    MouseArea {
                        id: tabMouseArea
                        anchors.fill: parent
                        hoverEnabled: true
                        ToolTip {
                            visible: tabMouseArea.containsMouse
                            text: tabsModel.get(index) ? tabsModel.get(index).name : ""
                        }
                        onClicked: mainWindow.switchToTab(index)
                    }
                    Text {
                        anchors.centerIn: parent
                        text: model.icon ? model.icon : (model.name ? model.name.charAt(0) : "")
                        font.pixelSize: 24
                        color: currentIndex === index ? "#fff"
                            : (currentTheme === "dark" ? "#ccc" : "#222")
                    }
                }
            }
            // "+" button
            Rectangle {
                width: 48; height: 48
                radius: 24
                color: "transparent"
                border.width: 1
                border.color: currentTheme === "dark" ? "#444" : "#bbb"
                MouseArea {
                    anchors.fill: parent
                    onClicked: mainWindow.addTab()
                }
                Text {
                    anchors.centerIn: parent
                    text: "+"
                    font.pixelSize: 28
                    color: "#888"
                }
            }
        }
        // ---- Spacer above the toggle ----
        Item {
            Layout.fillHeight: true
        }
        // ---- Vertically centered toggle button ----
        Button {
            id: toggleButton
            icon.name: mainWindow.sidebarVisible ? "go-previous" : "go-next"
            width: 30
            height: 30
            Layout.alignment: Qt.AlignHCenter
            ToolTip {
                text: mainWindow.sidebarVisible ? "Hide Sidebar40" : "Show Sidebar40"
            }
            onClicked: mainWindow.sidebarVisible = !mainWindow.sidebarVisible
        }
        // ---- Spacer below the toggle ----
        Item {
            Layout.fillHeight: true
        }
        // ---- Bottom action buttons ----
        Column {
            Layout.alignment: Qt.AlignHCenter
            spacing: 12
            Button {
                icon.name: "settings"
                onClicked: sidebar.openSettings()
                width: 40; height: 40
            }
            Button {
                icon.name: "help"
                onClicked: aboutDialog.open()
                width: 40; height: 40
            }
            Button {
                icon.name: "application-exit"
                onClicked: Qt.quit()
                width: 40; height: 40
            }
        }
    }
    // --- Dialogs ---
    SettingsDialog {
        id: settingsDialog
        controller: appController
    }
    Dialog {
        id: aboutDialog
        title: "About WhatsApp Desktop"
        standardButtons: Dialog.Ok
        Text {
            text: "WhatsApp Desktop\nVersion 1.0\n\nBuilt with Qt and Rust"
            horizontalAlignment: Text.AlignHCenter
        }
    }
}
