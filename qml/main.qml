import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import QtWebEngine
import QtQuick.Dialogs
// Namespaced: Qt.labs.platform also defines Menu/MenuItem, which would clash
// with the QtQuick.Controls ones used below.
import Qt.labs.platform 1.1 as Platform
import AppController
import "components" as Components
import "styles"

ApplicationWindow {
    id: mainWindow
    font.family: "Fira Code"
    font.pointSize: 10
    visible: !appController.start_minimized
    visibility: "Maximized"
    title: totalUnread > 0 ? "WhatsApp Desktop - Multi Account (" + totalUnread + ")" : "WhatsApp Desktop - Multi Account"
    onClosing: function (close) {
        if (appController.close_to_tray) {
            close.accepted = false;
            mainWindow.toggleVisible();
            return;
        }
        if (appController.clear_cache_on_exit) {
            for (var i = 0; i < tabsModel.count; i++) {
                var webTab = stackLayout.children[i];
                if (webTab && webTab.clearCache) {
                    webTab.clearCache();
                }
            }
        }
    }
    property alias controller: appController
    property string currentTheme: "system"
    property var downloadedFiles: []
    property string lastDownloadedFile: ""
    property string webProfilePath: "WhatsApp-QT-Profile"
    property bool sidebarVisible: true
    // Guards against the maximized startup size overwriting a saved windowed one
    // before load_settings() has had a chance to restore it.
    property bool geometryReady: false

    // Sum of every tab's unread count; drives the window title and tray tooltip.
    property int totalUnread: 0

    ListModel {
        id: tabsModel
        // A closing tab never emits unreadCountChanged, so recount once its
        // delegate is actually gone.
        onCountChanged: Qt.callLater(mainWindow.updateUnread)
    }
    onWidthChanged: { propagateAppSize(); saveGeometry(); }
    onHeightChanged: { propagateAppSize(); saveGeometry(); }
    onXChanged: saveGeometry();
    onYChanged: saveGeometry();

    // Only a windowed geometry is worth remembering; maximized/fullscreen sizes are
    // whatever the screen happens to be.
    function saveGeometry() {
        if (!geometryReady || visibility !== Window.Windowed)
            return;
        appController.set_window_geometry(width, height, x, y);
    }

    function restoreGeometry() {
        if (appController.window_width <= 0 || appController.window_height <= 0) {
            geometryReady = true;   // nothing saved: keep the maximized default
            return;
        }
        mainWindow.visibility = Window.Windowed;
        // Un-maximizing is a round trip to the window manager, which lands its own
        // "normal" geometry afterwards and would clobber an assignment made here.
        // Applying on the next event loop pass wins that race.
        Qt.callLater(applySavedGeometry);
    }

    function applySavedGeometry() {
        mainWindow.width = appController.window_width;
        mainWindow.height = appController.window_height;
        mainWindow.x = appController.window_x;
        mainWindow.y = appController.window_y;
        geometryReady = true;
        console.log("Restored windowed geometry:", width + "x" + height, "at", x + "," + y);
    }

    // One place that switches tabs: the Rust-side property AND the visible stack.
    // stackLayout.currentIndex is not bound to appController.current_tab, so setting
    // only the property (as the old Ctrl+Tab shortcuts did) changed nothing on screen.
    function switchToTab(i) {
        if (i < 0 || i >= tabsModel.count)
            return;
        stackLayout.currentIndex = i;
        appController.set_current_tab(i);
    }

    // Tab naming lives in Sidebar (nextTabName/sanitizeTabName); this is the single
    // entry point both the "+" button and Ctrl+T go through.
    function addTab() {
        appController.add_tab(sidebar.sanitizeTabName(sidebar.nextTabName(), "WhatsApp"), "💬");
    }

    function closeCurrentTab() {
        if (tabsModel.count > 1)
            appController.remove_tab(stackLayout.currentIndex);
    }

    // Zoom is a single global setting, so it applies to every open tab: per-tab zoom
    // that resets on tab switch would be worse.
    function applyZoom(factor) {
        var z = Math.max(0.25, Math.min(3.0, factor));
        for (var i = 0; i < stackLayout.children.length; i++) {
            var tab = stackLayout.children[i];
            if (tab && tab.zoomFactor !== undefined)
                tab.zoomFactor = z;
        }
        appController.set_zoom_factor(z);
    }
    // Same iteration as applyZoom: stackLayout.children includes the Repeater
    // itself, which the unreadCount guard filters out.
    function updateUnread() {
        var total = 0;
        for (var i = 0; i < stackLayout.children.length; i++) {
            var tab = stackLayout.children[i];
            if (tab && tab.unreadCount !== undefined)
                total += tab.unreadCount;
        }
        totalUnread = total;
    }

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
            mainWindow.restoreGeometry();
            console.log("Initial tabsModel count: " + tabsModel.count);
            if (tabsModel.count === 0) {
                console.log("No tabs found, creating initial tab...");
                add_tab("WhatsApp #1", "💬");
            }
            // A whatsapp:/wa.me URL the OS handed us on the command line. There is
            // no single-instance IPC, so this only fires for the process the OS
            // launched; all it can do is bring that window forward.
            var deepLink = take_pending_deep_link();
            if (deepLink.length > 0) {
                console.log("Deep link at startup:", deepLink);
                mainWindow.showAndRaise();
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
                    onUnreadCountChanged: mainWindow.updateUnread()
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

    Platform.SystemTrayIcon {
        id: trayIcon
        visible: true
        // A file:// URL, resolved once by Rust. It is a property on AppController
        // rather than a bare context property because cxx-qt-lib 0.7 has no
        // QQmlContext::setContextProperty binding.
        icon.source: appController.tray_icon_path
        // ponytail: no badge icon asset exists in resources/icons, and compositing
        // one at runtime is more QML than it is worth. Tooltip + window title carry
        // the count on every platform with zero extra assets.
        tooltip: mainWindow.totalUnread > 0 ? "WhatsApp Desktop (" + mainWindow.totalUnread + " unread)" : "WhatsApp Desktop"

        onActivated: mainWindow.toggleVisible()

        menu: Platform.Menu {
            Platform.MenuItem {
                text: mainWindow.visible ? "Hide" : "Show"
                onTriggered: mainWindow.toggleVisible()
            }
            Platform.MenuSeparator {}
            Platform.MenuItem {
                text: "Mute notifications for 1 hour"
                enabled: appController.notifications_enabled
                onTriggered: appController.snooze_notifications(60)
            }
            Platform.MenuItem {
                text: "Mute notifications for 8 hours"
                enabled: appController.notifications_enabled
                onTriggered: appController.snooze_notifications(480)
            }
            Platform.MenuItem {
                text: "Unmute notifications"
                enabled: !appController.notifications_enabled
                onTriggered: appController.set_notifications_enabled(true)
            }
            Platform.MenuSeparator {}
            Platform.MenuItem {
                text: "Quit"
                onTriggered: Qt.quit()
            }
        }
    }

    function showAndRaise() {
        mainWindow.visible = true;
        mainWindow.raise();
        mainWindow.requestActivate();
    }

    function toggleVisible() {
        if (mainWindow.visible)
            mainWindow.visible = false;
        else
            mainWindow.showAndRaise();
    }

    Shortcut {
        sequence: "Ctrl+Tab"
        onActivated: mainWindow.switchToTab((stackLayout.currentIndex + 1) % tabsModel.count)
    }
    Shortcut {
        sequence: "Ctrl+Shift+Tab"
        onActivated: mainWindow.switchToTab((stackLayout.currentIndex - 1 + tabsModel.count) % tabsModel.count)
    }
    Shortcut {
        sequence: "Ctrl+T"
        onActivated: mainWindow.addTab()
    }
    Shortcut {
        sequence: "Ctrl+W"
        onActivated: mainWindow.closeCurrentTab()
    }
    Shortcut {
        sequence: "Ctrl+,"
        onActivated: sidebar.openSettings()
    }
    Shortcut {
        sequence: "Ctrl+Q"
        onActivated: Qt.quit()
    }
    Shortcut {
        sequence: "Ctrl+Shift+M"
        onActivated: appController.set_notifications_enabled(!appController.notifications_enabled)
    }

    // Ctrl+1..Ctrl+9 -> tab 0..8. Instantiator, not Repeater: Shortcut is a QtObject.
    Instantiator {
        model: 9
        delegate: Shortcut {
            sequence: "Ctrl+" + (index + 1)
            onActivated: mainWindow.switchToTab(index)
        }
    }

    // Zoom: Ctrl+= and Ctrl++ both zoom in, since the plus key needs Shift on most layouts.
    Shortcut {
        sequences: ["Ctrl+=", "Ctrl++"]
        onActivated: mainWindow.applyZoom(appController.zoom_factor + 0.1)
    }
    Shortcut {
        sequence: "Ctrl+-"
        onActivated: mainWindow.applyZoom(appController.zoom_factor - 0.1)
    }
    Shortcut {
        sequence: "Ctrl+0"
        onActivated: mainWindow.applyZoom(1.0)
    }
}
