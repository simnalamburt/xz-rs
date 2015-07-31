extern crate pkg_config;
extern crate tar;

use std::io;
use std::env;
use std::fs::File;
use std::path::Path;
use std::process::Command;
use pkg_config::find_library;
use tar::Archive;

fn main() {
    // Use system installed liblzma if it does exist
    if find_library("liblzma").is_ok() { return }

    // Otherwise, compile liblzma manually
    compile().unwrap_or_else(|e| panic!("{}", e));
}

fn compile() -> io::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let num_jobs = env::var("NUM_JOBS").unwrap();

    // tar -xvf xz-5.2.1.tar
    let tar = try!(File::open(Path::new("xz-5.2.1.tar")));
    let mut archive = Archive::new(tar);
    try!(archive.unpack(&out_dir));

    // cd xz-5.2.1
    try!(env::set_current_dir(&out_dir));
    try!(env::set_current_dir("xz-5.2.1"));

    // ./configure
    let ret = try!(Command::new("./configure")
        .args(&[
            "--disable-debug",
            "--disable-dependency-tracking",
            "--disable-silent-rules",
        ])
        .arg(&format!("--prefix={}", out_dir))
        .status());
    if !ret.success() { panic!("`./configure` failed with {}", ret); }

    // make install
    let ret = try!(Command::new("make")
        .arg(&format!("-j{}", num_jobs))
        .arg("install")
        .status());
    if !ret.success() { panic!("`make` failed with {}", ret); }

    println!("cargo:rustc-flags=-L native={}/lib", out_dir);
    println!("cargo:rustc-flags=-l lzma");

    Ok(())
}
