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
        controller.save_settings();
        // controller.set_theme(themeCombo.currentText.toLowerCase());
        controller.set_download_path(downloadPathField.text);
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

                    CheckBox {
                        text: "Clear cache on exit"
                        checked: false
                    }

                    CheckBox {
                        text: "Start minimized to tray"
                        checked: false
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
            downloadPathField.text = selectedFolder.toString().replace("file://", "");
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
