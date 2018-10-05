extern crate gcc;

use std::env;

fn main() {
    /*match env::var("HADOOP_HOME") {
        Ok(val) => {
            println!("cargo:rustc-link-search=native={}/lib/native", val);
        }
        Err(e) => {
            panic!("HADOOP_HOME shell environment must be set: {}", e);
        }
    }
    
    match env::var("JAVA_HOME") {
        Ok(val) => {
            println!("cargo:rustc-link-search=native={}/include/linux", val);
        }
        Err(e) => {
            panic!("HADOOP_HOME shell environment must be set: {}", e);
        }
    }*/
    println!("cargo:rustc-link-search=native=libhdfs3/lib")
}
