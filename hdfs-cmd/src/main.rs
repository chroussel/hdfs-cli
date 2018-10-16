#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate hdfs;
use clap::{App, Arg, SubCommand};
use std::env;
use std::io::prelude::*;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

fn main() {
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    env_logger::Builder::from_env(env).init();

    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let hadoop_install_path = env::var("HADOOP_INSTALL").unwrap_or(String::from(""));
    let config = matches
        .value_of("config")
        .unwrap_or(hadoop_install_path.as_str());

    let gateway = matches.value_of("gateway").take();

    if let Some(matches) = matches.subcommand_matches("ls") {
        let path = matches.value_of("PATH").unwrap();
        let hdfs_fs = hdfs::hdfs::get_hdfs(Path::new(config), gateway, None).unwrap();

        for file in hdfs_fs.ls(path).unwrap() {
            println!("{}", file.name)
        }
    }
}
