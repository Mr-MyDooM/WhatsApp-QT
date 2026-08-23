use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    // NOTE: every .qml file the app loads must be listed in `qml_files` below.
    // They are compiled into the binary via the Qt resource system, so a new .qml
    // file that is not listed here simply does not exist at runtime.
    CxxQtBuilder::new()
        // QtWebEngineQuick, for the WebEngineView used by qml/components/WebTab.qml
        // and for whatsappqt_webengine_init().
        .qt_module("WebEngineQuick")
        .qml_module(QmlModule {
            // Single-segment uri, so the existing bare `import AppController`
            // (no version) in the QML keeps working unchanged.
            uri: "AppController",
            rust_files: &["src/cxxqt_object.rs"],
            qml_files: &[
                "qml/main.qml",
                "qml/components/Sidebar.qml",
                "qml/components/WebTab.qml",
                "qml/components/SettingsDialog.qml",
                "qml/styles/AppStyle.qml",
            ],
            ..Default::default()
        })
        .cc_builder(|cc| {
            cc.include("include");
            cc.file("src/webengine_init.cpp");
        })
        .build();
}
