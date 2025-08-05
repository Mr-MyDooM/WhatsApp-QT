import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts
import QtWebEngine
import QtQuick.Dialogs  // Qt6 - no version number
import AppController
import "components"
import "styles"

ApplicationWindow {
    id: mainWindow
    visible: true
    // visibility: "FullScreen"
    visibility: "Maximized"
    // width: 1400
    // height: 900
    // minimumWidth: 800
    // minimumHeight: 600
    title: "WhatsApp Desktop - Multi Account"

    property alias controller: appController
    property var tabs: []
    property string currentTheme: "system"

    AppController {
        id: appController

        Component.onCompleted: {
            load_settings();
            // Add default tab if none exist
            if (tabs.length === 0) {
                add_tab("WhatsApp #1", "💬");
            }
        }

        onTab_added: function (name, icon) {
            var newTab = {
                name: name,
                icon: icon,
                url: "https://web.whatsapp.com/"
            };
            tabs.push(newTab);
            stackLayout.currentIndex = tabs.length - 1;
            tabsChanged();
        }

        onTab_removed: function (index) {
            if (tabs.length > 1) {
                tabs.splice(index, 1);
                if (stackLayout.currentIndex >= tabs.length) {
                    stackLayout.currentIndex = tabs.length - 1;
                }
                tabsChanged();
            }
        }

        onTab_renamed: function (index, newName) {
            if (index < tabs.length) {
                tabs[index].name = newName;
                tabsChanged();
            }
        }

        onTheme_changed: {
            currentTheme = appController.theme;
            applyTheme();
        }
    }

    function applyTheme() {
        switch (currentTheme) {
        case "dark":
            mainWindow.color = "#2d2d2d";
            break;
        case "light":
            mainWindow.color = "#ffffff";
            break;
        default:
            // system
            mainWindow.color = palette.window;
        }

        // Apply theme to all web views
        for (var i = 0; i < stackLayout.count; i++) {
            var item = stackLayout.children[i];
            if (item && item.children[0]) {
                var webView = item.children[0];
                if (webView.applyThemeCSS) {
                    webView.applyThemeCSS();
                }
            }
        }
    }

    // Main Layout - Row layout for sidebar + content
    RowLayout {
        anchors.fill: parent
        spacing: 0

        // Left Sidebar - Contains Menu and Tabs
        Rectangle {
            id: sidebar
            Layout.preferredWidth: 250
            Layout.fillHeight: true
            color: currentTheme === "dark" ? "#2d2d2d" : "#f0f0f0"
            border.color: currentTheme === "dark" ? "#555" : "#ccc"
            border.width: 1

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 5
                spacing: 5

                // Menu Section
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 200
                    color: currentTheme === "dark" ? "#3d3d3d" : "#ffffff"
                    border.color: currentTheme === "dark" ? "#555" : "#ddd"
                    border.width: 1
                    radius: 5

                    ScrollView {
                        anchors.fill: parent
                        anchors.margins: 10

                        ColumnLayout {
                            width: parent.width
                            spacing: 8

                            Text {
                                text: "Menu"
                                font.bold: true
                                font.pixelSize: 14
                                color: currentTheme === "dark" ? "#ffffff" : "#000000"
                                Layout.fillWidth: true
                            }

                            Rectangle {
                                Layout.fillWidth: true
                                Layout.preferredHeight: 1
                                color: currentTheme === "dark" ? "#555" : "#ddd"
                            }

                            // File Menu Items
                            Button {
                                Layout.fillWidth: true
                                text: "➕ New Tab (Ctrl+T)"
                                flat: true
                                onClicked: appController.add_tab("WhatsApp #" + (tabs.length + 1), "💬")

                                background: Rectangle {
                                    color: parent.hovered ? (currentTheme === "dark" ? "#555" : "#e0e0e0") : "transparent"
                                    radius: 3
                                }

                                contentItem: Text {
                                    text: parent.text
                                    color: currentTheme === "dark" ? "#ffffff" : "#000000"
                                    horizontalAlignment: Text.AlignLeft
                                    verticalAlignment: Text.AlignVCenter
                                }
                            }

                            Button {
                                Layout.fillWidth: true
                                text: "❌ Close Tab (Ctrl+W)"
                                flat: true
                                enabled: tabs.length > 1
                                onClicked: appController.remove_tab(stackLayout.currentIndex)

                                background: Rectangle {
                                    color: parent.hovered ? (currentTheme === "dark" ? "#555" : "#e0e0e0") : "transparent"
                                    radius: 3
                                }

                                contentItem: Text {
                                    text: parent.text
                                    color: parent.enabled ? (currentTheme === "dark" ? "#ffffff" : "#000000") : "#888888"
                                    horizontalAlignment: Text.AlignLeft
                                    verticalAlignment: Text.AlignVCenter
                                }
                            }

                            Button {
                                Layout.fillWidth: true
                                text: "⚙️ Settings (Ctrl+,)"
                                flat: true
                                onClicked: settingsDialog.open()

                                background: Rectangle {
                                    color: parent.hovered ? (currentTheme === "dark" ? "#555" : "#e0e0e0") : "transparent"
                                    radius: 3
                                }

                                contentItem: Text {
                                    text: parent.text
                                    color: currentTheme === "dark" ? "#ffffff" : "#000000"
                                    horizontalAlignment: Text.AlignLeft
                                    verticalAlignment: Text.AlignVCenter
                                }
                            }

                            Button {
                                Layout.fillWidth: true
                                text: "ℹ️ About"
                                flat: true
                                onClicked: aboutDialog.open()

                                background: Rectangle {
                                    color: parent.hovered ? (currentTheme === "dark" ? "#555" : "#e0e0e0") : "transparent"
                                    radius: 3
                                }

                                contentItem: Text {
                                    text: parent.text
                                    color: currentTheme === "dark" ? "#ffffff" : "#000000"
                                    horizontalAlignment: Text.AlignLeft
                                    verticalAlignment: Text.AlignVCenter
                                }
                            }

                            Button {
                                Layout.fillWidth: true
                                text: "🚪 Quit (Ctrl+Q)"
                                flat: true
                                onClicked: Qt.quit()

                                background: Rectangle {
                                    color: parent.hovered ? "#ff4444" : "transparent"
                                    radius: 3
                                }

                                contentItem: Text {
                                    text: parent.text
                                    color: parent.hovered ? "#ffffff" : (currentTheme === "dark" ? "#ffffff" : "#000000")
                                    horizontalAlignment: Text.AlignLeft
                                    verticalAlignment: Text.AlignVCenter
                                }
                            }
                        }
                    }
                }

                // Tabs Section
                Rectangle {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    color: currentTheme === "dark" ? "#3d3d3d" : "#ffffff"
                    border.color: currentTheme === "dark" ? "#555" : "#ddd"
                    border.width: 1
                    radius: 5

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: 10
                        spacing: 5

                        RowLayout {
                            Layout.fillWidth: true

                            Text {
                                text: "Tabs"
                                font.bold: true
                                font.pixelSize: 14
                                color: currentTheme === "dark" ? "#ffffff" : "#000000"
                                Layout.fillWidth: true
                            }

                            Button {
                                Layout.preferredWidth: 25
                                Layout.preferredHeight: 25
                                text: "+"
                                font.pixelSize: 16
                                onClicked: appController.add_tab("WhatsApp #" + (tabs.length + 1), "💬")

                                background: Rectangle {
                                    color: parent.hovered ? (currentTheme === "dark" ? "#555" : "#e0e0e0") : "transparent"
                                    border.color: currentTheme === "dark" ? "#666" : "#ccc"
                                    border.width: 1
                                    radius: 3
                                }
                            }
                        }

                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 1
                            color: currentTheme === "dark" ? "#555" : "#ddd"
                        }

                        ScrollView {
                            Layout.fillWidth: true
                            Layout.fillHeight: true

                            ColumnLayout {
                                width: parent.width
                                spacing: 2

                                Repeater {
                                    model: tabs.length

                                    Rectangle {
                                        Layout.fillWidth: true
                                        Layout.preferredHeight: 60
                                        color: stackLayout.currentIndex === index ? (currentTheme === "dark" ? "#555" : "#e3f2fd") : "transparent"
                                        border.color: currentTheme === "dark" ? "#666" : "#ddd"
                                        border.width: stackLayout.currentIndex === index ? 1 : 0
                                        radius: 5

                                        MouseArea {
                                            anchors.fill: parent
                                            onClicked: {
                                                stackLayout.currentIndex = index;
                                                appController.set_current_tab(index);
                                            }
                                        }

                                        ColumnLayout {
                                            anchors.fill: parent
                                            anchors.margins: 8
                                            spacing: 4

                                            RowLayout {
                                                Layout.fillWidth: true

                                                Text {
                                                    text: tabs[index] ? tabs[index].icon : ""
                                                    font.pixelSize: 18
                                                }

                                                Text {
                                                    Layout.fillWidth: true
                                                    text: tabs[index] ? tabs[index].name : ""
                                                    color: currentTheme === "dark" ? "#ffffff" : "#000000"
                                                    font.pixelSize: 12
                                                    font.bold: stackLayout.currentIndex === index
                                                    elide: Text.ElideRight
                                                }

                                                Button {
                                                    Layout.preferredWidth: 20
                                                    Layout.preferredHeight: 20
                                                    text: "×"
                                                    font.pixelSize: 12
                                                    visible: tabs.length > 1

                                                    onClicked: {
                                                        //mouse.accepted = true;
                                                        appController.remove_tab(index);
                                                    }

                                                    background: Rectangle {
                                                        color: parent.hovered ? "#ff4444" : "transparent"
                                                        radius: 10
                                                    }

                                                    contentItem: Text {
                                                        text: parent.text
                                                        color: parent.hovered ? "#ffffff" : (currentTheme === "dark" ? "#ffffff" : "#000000")
                                                        horizontalAlignment: Text.AlignHCenter
                                                        verticalAlignment: Text.AlignVCenter
                                                    }
                                                }
                                            }

                                            TextInput {
                                                Layout.fillWidth: true
                                                text: tabs[index] ? tabs[index].name : ""
                                                color: currentTheme === "dark" ? "#ffffff" : "#000000"
                                                selectByMouse: true
                                                font.pixelSize: 10
                                                visible: false  // Hidden by default, could be shown on double-click

                                                onEditingFinished: {
                                                    if (tabs[index] && text !== tabs[index].name) {
                                                        appController.rename_tab(index, text);
                                                    }
                                                    visible = false;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Main Content Area
        StackLayout {
            id: stackLayout
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: 0

            Repeater {
                model: tabs.length

                Item {
                    WebEngineView {
                        id: webView
                        anchors.fill: parent

                        profile: WebEngineProfile {
                            id: webProfile
                            storageName: "whatsapp_tab_" + index
                            offTheRecord: false
                            httpUserAgent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
                            persistentCookiesPolicy: WebEngineProfile.ForcePersistentCookies
                        }

                        Component.onCompleted: {
                            var baseUrl = "https://web.whatsapp.com/";
                            var uniqueParam = "?tab_id=" + index + "&timestamp=" + Date.now();
                            url = baseUrl + uniqueParam;
                        }

                        onNewWindowRequested: function (request) {
                            Qt.openUrlExternally(request.requestedUrl);
                        }

                        onLoadingChanged: function (loadRequest) {
                            if (loadRequest.status === WebEngineView.LoadSucceededStatus) {
                                applyThemeCSS();
                            }
                        }

                        function applyThemeCSS() {
                            var css = appController.get_theme_css();
                            if (css.length > 0) {
                                var script = `
                                    (function() {
                                        var existingStyle = document.getElementById('app-theme-style');
                                        if (existingStyle) {
                                            existingStyle.remove();
                                        }

                                        var style = document.createElement('style');
                                        style.id = 'app-theme-style';
                                        style.textContent = \`${css}\`;
                                        document.head.appendChild(style);
                                    })();
                                `;

                                runJavaScript(script);
                            }
                        }

                        Connections {
                            target: appController
                            function onApply_theme_css(css_code) {
                                var script = `
                                    (function() {
                                        var existingStyle = document.getElementById('app-theme-style');
                                        if (existingStyle) {
                                            existingStyle.remove();
                                        }

                                        var style = document.createElement('style');
                                        style.id = 'app-theme-style';
                                        style.textContent = \`${css_code}\`;
                                        document.head.appendChild(style);
                                    })();
                                `;

                                runJavaScript(script);
                            }
                        }
                    }
                }
            }
        }
    }

    // Keep keyboard shortcuts active
    Shortcut {
        sequence: "Ctrl+T"
        onActivated: appController.add_tab("WhatsApp #" + (tabs.length + 1), "💬")
    }

    Shortcut {
        sequence: "Ctrl+W"
        enabled: tabs.length > 1
        onActivated: appController.remove_tab(stackLayout.currentIndex)
    }

    Shortcut {
        sequence: "Ctrl+,"
        onActivated: settingsDialog.open()
    }

    Shortcut {
        sequence: "Ctrl+Q"
        onActivated: Qt.quit()
    }

    // Settings Dialog
    SettingsDialog {
        id: settingsDialog
        controller: appController
    }

    // About Dialog
    Dialog {
        id: aboutDialog
        title: "About WhatsApp Desktop"
        standardButtons: Dialog.Ok

        Text {
            text: "WhatsApp Desktop\nVersion 1.0\n\nBuilt with Qt and Rust\nEmbeds WhatsApp Web for multi-account support"
            horizontalAlignment: Text.AlignHCenter
        }
    }
}
