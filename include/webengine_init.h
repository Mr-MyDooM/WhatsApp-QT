#pragma once

// QtWebEngineQuick::initialize() must run BEFORE the QGuiApplication is
// constructed, otherwise every WebEngineView aborts at runtime with
// "QtWebEngineQuick::initialize() must be called before the construction of
// QGuiApplication". There is no cxx-qt-lib binding for it, so this is the
// smallest possible C++ shim, called first thing in main().
//
// Deliberately NOT inline: cxx links against a real symbol for this, and an
// inline-only definition is not guaranteed to emit one.
void whatsappqt_webengine_init();
