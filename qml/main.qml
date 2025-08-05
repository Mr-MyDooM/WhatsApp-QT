import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import QtWebEngine
import QtQuick.Dialogs
import AppController
import "components" as Components
import "styles"

ApplicationWindow {
    id: mainWindow
    font.family: "Fira Code"
    font.pointSize: 10
    visible: true
    visibility: "Maximized"
    title: "WhatsApp Desktop - Multi Account"
    property alias controller: appController
    property string currentTheme: "system"
    property var downloadedFiles: []
    property string lastDownloadedFile: ""
    property string webProfilePath: "WhatsApp-QT-Profile"
    property bool sidebarVisible: true

    ListModel {
        id: tabsModel
    }
    onWidthChanged: propagateAppSize();
    onHeightChanged: propagateAppSize();
    function propagateAppSize() {
        for (var i = 0; i < tabsModel.count; i++) {
            var item = stackLayout.children[i];
            var delegateItem = stackLayout.children[i];
            if (delegateItem && delegateItem.children.length > 0) {
                var webView = delegateItem.children[0];
                if (webView && webView.injectAppSize) {
                    webView.injectAppSize(mainWindow.width, mainWindow.height);
                }
            }
        }
    }

    AppController {
        id: appController
        Component.onCompleted: {
            console.log("Loading initial settings...");
            load_settings();
            currentTheme = appController.theme;
            applyTheme();
            console.log("Initial tabsModel count: " + tabsModel.count);
            if (tabsModel.count === 0) {
                console.log("No tabs found, creating initial tab...");
                add_tab("WhatsApp #1", "💬");
            }
        }

        onTab_added: function (name, icon) {
            console.log("QML onTab_added: Adding tab to model ->", name);
            tabsModel.append({
                name: name,
                icon: icon,
                // url: "https://browserleaks.com/http2"
                url: "https://web.whatsapp.com"
            });
            stackLayout.currentIndex = tabsModel.count - 1;
        }
        onTab_removed: function (index) {
            if (tabsModel.count > 1) {
                tabsModel.remove(index);
                if (stackLayout.currentIndex >= tabsModel.count) {
                    stackLayout.currentIndex = tabsModel.count - 1;
                }
            }
        }

        onTab_renamed: function (index, newName) {
            if (index < tabsModel.count) {
                var oldIcon = tabsModel.get(index).icon;
                var oldUrl = tabsModel.get(index).url;
                tabsModel.set(index, {name: newName, icon: oldIcon, url: oldUrl});
            }
        }
        onTheme_changed: {
            console.log("Theme changed signal received in QML, new theme:", appController.theme);

            var newTheme = appController.theme;
            if (currentTheme !== newTheme) {
                currentTheme = newTheme;
                sidebar.currentTheme = currentTheme;
                applyTheme();
            }

        }
    }

    function applyTheme() {
        console.log("Applying theme:", currentTheme);

        switch (currentTheme) {
            case "dark":
                mainWindow.color = "#2d2d2d";
                break;
            case "light":
                mainWindow.color = "#ffffff";
                break;
            default:
                mainWindow.color = palette.window;
        }

        // Generate CSS and apply to all web views in one go
        var css = appController.get_theme_css();
        if (css && css.length > 0) {
            console.log("Applying CSS to", tabsModel.count, "web views");
            for (var i = 0; i < tabsModel.count; i++) {
                var delegateItem = stackLayout.children[i];
                if (delegateItem && delegateItem.applyThemeCSS) {
                    console.log("Applying theme to web view", i);
                    delegateItem.applyThemeCSS();
                }
            }
        }
    }


    RowLayout {
        anchors.fill: parent
        spacing: 0

        Components.Sidebar {
            id: sidebar
            visible: sidebarVisible
            currentTheme: mainWindow.currentTheme
            appController: appController
            stackLayout: stackLayout
            tabsModel: tabsModel
            Layout.preferredWidth: sidebarVisible ? 64 : 0
            Layout.fillHeight: true
        }

        // Tab context menu
        Menu {
            id: tabContextMenu
            property int contextIndex: -1

            MenuItem {
                text: "Rename"
                onTriggered: {
                    // Implement rename logic
                }
            }
            MenuItem {
                text: "Close"
                onTriggered: appController.remove_tab(tabContextMenu.contextIndex)
            }
        }

        // In tab delegate:
        MouseArea {
            acceptedButtons: Qt.LeftButton | Qt.RightButton
            onClicked: {
                if (mouse.button === Qt.RightButton) {
                    tabContextMenu.contextIndex = index;
                    tabContextMenu.popup();
                }
            }
        }
        // Main content area
        StackLayout {
            id: stackLayout
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: 0
            Repeater {
                model: tabsModel
                delegate: Components.WebTab
                {
                    id: webTabDelegate
                    index: model.index
                    url: model.url
                }
            }
        }
    }

    MouseArea {
        id: showSidebarMouseArea
        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        width: 30
        height: 150
        hoverEnabled: true
        z: 1000
        property bool hovered: false
        onEntered: hovered = true
        onExited: hovered = false

        // Only show when sidebar is hidden and mouse is over this area
        visible: !sidebarVisible
        Button {
            id: showSidebarBtn
            width: 30; height: 30
            anchors.centerIn: parent
            visible: showSidebarMouseArea.hovered
            icon.name: "go-next"
            ToolTip {
                text: "Show Sidebar"
            }
            onClicked: sidebarVisible = true
        }
    }

    Shortcut {
        sequence: "Ctrl+Tab"
        onActivated: {
            let next = (stackLayout.currentIndex + 1) % tabsModel.count
            appController.set_current_tab(next)
        }
    }
    Shortcut {
        sequence: "Ctrl+Shift+Tab"
        onActivated: {
            let prev = (stackLayout.currentIndex - 1 + tabsModel.count) % tabsModel.count
            appController.set_current_tab(prev)
        }
    }
}
