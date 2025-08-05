// In qml/components/WebTab.qml
import QtQuick 2.15
import QtQuick.Window 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts
import QtWebEngine
import QtQuick.Dialogs
import AppController
import QtCore
import "."
import "../styles"

Item {
    id: webTabRoot
    property int index: 0
    property string url: ""
    WebEngineProfilePrototype {
        id: profilePrototype
        storageName: "whatsapp_tab_" + webTabRoot.index
        // persistentStoragePath: StandardPaths.writableLocation(StandardPaths.AppDataLocation) + "/WhatsApp-QT-Profile/" + getStorageName()
        persistentStoragePath: StandardPaths.writableLocation(StandardPaths.AppDataLocation) + "/WhatsApp-QT-Profile/whatsapp_tab_" + webTabRoot.index
        persistentCookiesPolicy: WebEngineProfile.ForcePersistentCookies

    }

    // Function to apply CSS, correctly placed in the parent scope
    function applyThemeCSS() {
        console.log("Applying theme to WebTab", webTabRoot.index);

        var css = mainWindow.controller.get_theme_css();
        if (css && css.length > 0) {
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
            webView.runJavaScript(script);
        }
    }

    function getStorageName() {
        return "whatsapp_tab_" + webTabRoot.index;
    }

    function injectAppSize(w, h) {
        var js = `
        window.dispatchEvent(new Event("resize"));
    `;
        webView.runJavaScript(js);
    }

    WebEngineView {
        id: webView
        anchors.fill: parent
        profile: profilePrototype.instance()
        url: webTabRoot.url
        Component.onCompleted: {
            console.log("WebTab created for index:", webTabRoot.index, "loading url:", webTabRoot.url);
            // profile.httpUserAgent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";
            // profile.httpUserAgent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.82 Safari/537.36 WhatsApp/2.2134.10";
            profile.httpUserAgent = mainWindow.controller.get_user_agent();
            console.log("Using User Agent:", profile.httpUserAgent);
        }
        onLoadingChanged: function (loadRequest) {
            console.log("Load status for tab", webTabRoot.index, ":", loadRequest.status);
            if (loadRequest.errorString) {
                console.error("Load error in tab", webTabRoot.index, ":", loadRequest.errorString);
            }
            if (loadRequest.status === WebEngineView.LoadSucceededStatus) {
                console.log("Page loaded successfully, applying theme...");
                // applyThemeTimer.start();
            }
        }
        onNewWindowRequested: function (request) {
            Qt.openUrlExternally(request.requestedUrl);
        }
    }
    // Connections {
    //     target: appController
    //
    //     function onApply_theme_css(css_code) {
    //         applyThemeCSS();
    //     }
    // }

}
