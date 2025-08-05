pragma Singleton
import QtQuick 2.15

QtObject {
    // Theme colors
    property color backgroundColor: "#ffffff"
    property color primaryColor: "#25d366"
    property color secondaryColor: "#128c7e"
    property color textColor: "#000000"

    // Dark theme colors
    property color darkBackgroundColor: "#2d2d2d"
    property color darkTextColor: "#ffffff"

    // Sizes
    property int tabHeight: 40
    property int buttonRadius: 4
    property int marginSize: 10
}
