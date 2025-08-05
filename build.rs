fn main() {
    println!("cargo:rerun-if-changed=resources.qrc");
    println!("cargo:rerun-if-changed=qml/");
    println!("Qt Resource System is disabled - using include_str! approach");
}
