extern crate "pkg-config" as pkg_config;

fn main() {
    match pkg_config::find_library("liblzma") {
        Ok(_) => return,
        Err(..) => unimplemented!()
    }
}
