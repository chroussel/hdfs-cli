extern crate gcc;

use std::env;

fn main() {
    println!("cargo:rustc-link-search=native=libhdfs3/lib")
}
