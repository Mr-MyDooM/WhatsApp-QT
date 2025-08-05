fn check_updates(&self) {
    let response = reqwest::blocking::get(
        "https://api.github.com/repos/mr.mydoom/whatsapp-qt/releases/latest",
    )
    .unwrap()
    .text()
    .unwrap();

    let latest_ver = json::parse(&response).unwrap()["tag_name"]
        .as_str()
        .unwrap();

    if latest_ver != env!("CARGO_PKG_VERSION") {
        self.update_available(QString::from(latest_ver));
    }
}
