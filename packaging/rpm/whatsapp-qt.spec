Name:           whatsapp-qt
Version:        0.1.0
Release:        1%{?dist}
Summary:        WhatsApp desktop client for Linux
License:        MIT
URL:            https://github.com/mrdoom/whatsapp-qt
BuildArch:      x86_64

# Runtime dependencies (Qt6 + WebEngine). No GTK/appindicator: the tray is
# Qt.labs.platform.SystemTrayIcon in QML, not the tray-item crate.
# NOTE: Fedora naming. openSUSE uses qt6-base / qt6-declarative / qt6-webengine.
Requires:       qt6-qtbase
Requires:       qt6-qtdeclarative
Requires:       qt6-qtwebengine

%description
A Linux-first, privacy-friendly WhatsApp desktop client built with
Rust and Qt. Supports multiple accounts via browser-like tabs,
light/dark/system themes, and system tray integration.

%install
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_datadir}/whatsapp-qt
mkdir -p %{buildroot}%{_datadir}/applications
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/512x512/apps

install -m 755 %{_sourcedir}/WhatsApp-QT          %{buildroot}%{_bindir}/WhatsApp-QT
cp -r          %{_sourcedir}/qml                   %{buildroot}%{_datadir}/whatsapp-qt/qml
cp -r          %{_sourcedir}/resources             %{buildroot}%{_datadir}/whatsapp-qt/resources
install -m 644 %{_sourcedir}/WhatsApp-QT.desktop   %{buildroot}%{_datadir}/applications/io.github.mrdoom.WhatsAppQT.desktop
install -m 644 %{_sourcedir}/tray.png              %{buildroot}%{_datadir}/icons/hicolor/512x512/apps/io.github.mrdoom.WhatsAppQT.png

%files
%{_bindir}/WhatsApp-QT
%{_datadir}/whatsapp-qt/
%{_datadir}/applications/io.github.mrdoom.WhatsAppQT.desktop
%{_datadir}/icons/hicolor/512x512/apps/io.github.mrdoom.WhatsAppQT.png

%post
update-desktop-database -q %{_datadir}/applications 2>/dev/null || true
gtk-update-icon-cache -q %{_datadir}/icons/hicolor 2>/dev/null || true

%changelog
* Mon Apr 21 2026 MJ <mritunjay.rajput@armakuni.com> - 0.1.0-1
- Initial release
