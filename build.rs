fn main() {
    println!("cargo::rerun-if-changed=vendor");
    println!("cargo::rustc-link-arg=-Wl,-Tlink.ld");
    println!("cargo::rustc-link-arg=-nostartfiles");
    println!("cargo::rustc-link-arg=-nodefaultlibs");
    println!("cargo::rustc-link-arg=-Wl,--no-warn-rwx-segments");
}
