#![feature(env, process)]
#![feature(old_path)]

extern crate "pkg-config" as pkg_config;

use std::env;
use std::process::Command;

fn main() {
    // Cross compile not supported yet. See #8
    if env::var("TARGET") != env::var("HOST") { unimplemented!() }

    // Use system installed liblzma if it exists, compile it manually otherwise.
    match pkg_config::find_library("liblzma") {
        Ok(_) => return,
        Err(_) => ()
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let num_jobs = env::var("NUM_JOBS").unwrap();

    // cd xz
    env::set_current_dir(&Path::new("xz")).unwrap();

    let ret = Command::new("./configure")
        .args(&[
              "--disable-debug",
              "--disable-dependency-tracking",
              "--disable-silent-rules",
        ])
        .arg(&format!("--prefix={}", out_dir))
        .status().unwrap().success();
    assert!(ret);

    let ret = Command::new("make")
        .arg(&format!("-j{}", num_jobs))
        .arg("install")
        .status().unwrap().success();
    assert!(ret);

    println!("cargo:rustc-flags=-L native={}/lib", out_dir);
    println!("cargo:rustc-flags=-l lzma");
}
