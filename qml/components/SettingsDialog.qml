import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import QtQuick.Dialogs
import QtCore
import "."
import "../styles"

Dialog {
    id: settingsDialog
    title: "Settings"
    width: 600
    height: 550
    standardButtons: Dialog.Ok | Dialog.Cancel

    property var controller

    onAccepted: {
        var selectedTheme = themeCombo.currentText.toLowerCase();
        console.log("Applying theme:", selectedTheme);
        controller.set_theme(selectedTheme);

        // Path comes from the folder picker (field is readOnly), so an empty
        // string is the only bad value reachable here.
        var path = downloadPathField.text.trim();
        if (path.length > 0)
            controller.set_download_path(path);
        else
            console.warn("Download path empty, keeping previous value");

        // save_settings() must run last, it serialises whatever the setters above wrote.
        controller.save_settings();
    }

    ScrollView {
        anchors.fill: parent
        anchors.margins: 20
        contentWidth: availableWidth

        ColumnLayout {
            width: parent.width
            spacing: 20

            GroupBox {
                title: "Appearance"
                Layout.fillWidth: true

                ColumnLayout {
                    anchors.fill: parent

                    RowLayout {
                        Label {
                            text: "Theme:"
                        }
                        ComboBox {
                            id: themeCombo
                            Layout.fillWidth: true
                            model: ["System", "Light", "Dark"]

                            Component.onCompleted: {
                                var theme = controller.theme;
                                if (theme === "light")
                                    currentIndex = 1;
                                else if (theme === "dark")
                                    currentIndex = 2;
                                else
                                    currentIndex = 0;
                            }
                        }
                    }
                }
            }

            GroupBox {
                title: "Downloads"
                Layout.fillWidth: true

                ColumnLayout {
                    anchors.fill: parent

                    RowLayout {
                        Label {
                            text: "Download Path:"
                        }
                        TextField {
                            id: downloadPathField
                            Layout.fillWidth: true
                            text: controller.download_path
                            placeholderText: "Choose download folder..."
                            readOnly: true
                        }
                        Button {
                            text: "Browse..."
                            onClicked: folderDialog.open()
                        }
                    }
                }
            }

            GroupBox {
                title: "Notifications"
                Layout.fillWidth: true

                ColumnLayout {
                    anchors.fill: parent

                    CheckBox {
                        id: enableNotificationsCheck
                        text: "Enable notifications"
                        Component.onCompleted: checked = controller.notifications_enabled
                        onClicked: controller.set_notifications_enabled(checked)
                    }

                    CheckBox {
                        id: messageNotificationsCheck
                        text: "Show message notifications"
                        enabled: enableNotificationsCheck.checked
                        Component.onCompleted: checked = controller.show_message_notifications
                        onClicked: controller.set_show_message_notifications(checked)
                    }

                    CheckBox {
                        id: callNotificationsCheck
                        text: "Show call notifications"
                        enabled: enableNotificationsCheck.checked
                        Component.onCompleted: checked = controller.show_call_notifications
                        onClicked: controller.set_show_call_notifications(checked)
                    }

                    CheckBox {
                        id: soundEnabledCheck
                        text: "Enable notification sounds"
                        enabled: enableNotificationsCheck.checked
                        Component.onCompleted: checked = controller.notification_sound_enabled
                        onClicked: controller.set_notification_sound_enabled(checked)
                    }

                    CheckBox {
                        id: showSenderCheck
                        text: "Show sender information"
                        enabled: enableNotificationsCheck.checked
                        Component.onCompleted: checked = controller.show_sender
                        onClicked: controller.set_show_sender(checked)
                    }

                    RowLayout {
                        Button {
                            text: "Test Notification"
                            enabled: enableNotificationsCheck.checked
                            onClicked: controller.test_notification()
                        }
                    }
                }
            }

            GroupBox {
                title: "Privacy & Security"
                Layout.fillWidth: true

                ColumnLayout {
                    anchors.fill: parent

                    // These two follow the same property + setter pattern as the
                    // notification checkboxes above, but are guarded the way
                    // Sidebar.qml guards set_current_tab: the AppController
                    // properties/setters are not on the Rust side yet, so an
                    // unguarded call would throw. They persist as soon as it lands.
                    CheckBox {
                        id: clearCacheOnExitCheck
                        text: "Clear cache on exit"
                        Component.onCompleted: checked = controller && controller.clear_cache_on_exit === true
                        onClicked: {
                            if (controller && controller.set_clear_cache_on_exit)
                                controller.set_clear_cache_on_exit(checked);
                            else
                                console.warn("AppController.set_clear_cache_on_exit missing; setting not persisted");
                        }
                    }

                    CheckBox {
                        id: startMinimizedCheck
                        text: "Start minimized to tray"
                        Component.onCompleted: checked = controller && controller.start_minimized === true
                        onClicked: {
                            if (controller && controller.set_start_minimized)
                                controller.set_start_minimized(checked);
                            else
                                console.warn("AppController.set_start_minimized missing; setting not persisted");
                        }
                    }

                    CheckBox {
                        id: closeToTrayCheck
                        text: "Close to tray instead of quitting"
                        Component.onCompleted: checked = controller && controller.close_to_tray === true
                        onClicked: {
                            if (controller && controller.set_close_to_tray)
                                controller.set_close_to_tray(checked);
                            else
                                console.warn("AppController.set_close_to_tray missing; setting not persisted");
                        }
                    }
                }
            }

            Item {
                Layout.fillHeight: true
            } // Spacer
        }

    }
    FolderDialog {
        id: folderDialog
        title: "Choose Download Folder"
        currentFolder: {
            var downloadPaths = StandardPaths.standardLocations(StandardPaths.DownloadLocation);
            if (downloadPaths.length > 0) {
                return downloadPaths[0];
            }

            var homePaths = StandardPaths.standardLocations(StandardPaths.HomeLocation);
            if (homePaths.length > 0) {
                return homePaths[0] + "/Downloads";
            }

            // Final fallback - construct path manually
            return "file://" + StandardPaths.writableLocation(StandardPaths.HomeLocation) + "/Downloads";
        }

        onAccepted: {
            // ponytail: a native folder picker cannot return garbage, so a
            // null/empty check is the whole validation this needs.
            if (!selectedFolder)
                return;
            var picked = selectedFolder.toString().replace("file://", "");
            if (picked.length > 0)
                downloadPathField.text = picked;
        }
    }


    Connections {

        target: controller

        function onSave_failed(error) {

            settingsDialog.close();

            messageDialog.text = "Error saving settings: " + error;
            messageDialog.open();
        }
    }


    MessageDialog {
        id: messageDialog
        title: "Error"

        text: ""
    }
}
