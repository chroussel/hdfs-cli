#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;
extern crate dirs;
extern crate env_logger;
extern crate hdfs;
extern crate serde;
extern crate serde_json;
extern crate toml;
extern crate walk;

mod config;
mod err;
mod walk_hdfs;

use clap::App;
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::PathBuf;
use walk::walk::{DirEntryTrait, FileSystem, MetadataTrait};

macro_rules! try_or_print {
    ($e: expr) => {
        match ($e) {
            Ok(value) => value,
            Err(e) => println!("{:?}", e),
        }
    };
}

fn ls(config_path: PathBuf, gateway: Option<&str>, path: PathBuf) {
    //let hdfs_fs = hdfs::hdfs::get_hdfs(config_path, gateway, None).unwrap();
    //let fs = walk_hdfs::HdfsFileSystem::new(&hdfs_fs);
    let fs = walk::linuxfs::LinuxFS::default();

    let walk: Vec<Result<_, _>> = walk::walk::WalkBuilder::new(&fs)
        .with_path(path)
        .build()
        .unwrap()
        .collect();

    let mut walk = walk;
    if walk.len() == 1 {
        try_or_print!(walk.pop().unwrap().map(|item| {
            print_item(&fs, &item);
        }));
    } else {
        for item in walk {
            try_or_print!(item.map(|i| print_item(&fs, &i)));
        }
    }
}

fn print_item(fs: &walk::linuxfs::LinuxFS, item: &walk::walk::WalkItem, bool print_path) {
    if item.is_dir() {
        println!("{}:", item.path().display());
        print_dir(&fs, &item.path());
    } else if print_path {
    }    else {
        println!("{}", file_name(&item.path()));
    }
}

fn print_dir(fs: &walk::linuxfs::LinuxFS, path: &PathBuf) {
    for i in fs.read_dir(path).unwrap() {
        let i = i.unwrap();
        print!("{} ", file_name(&i.path()));
    }
    println!()
}

fn file_name(path: &PathBuf) -> &str {
    path.file_name().unwrap().to_str().unwrap()
}

fn text(config_path: PathBuf, gateway: Option<&str>, path: PathBuf) {
    let hdfs_fs = hdfs::hdfs::get_hdfs(config_path, gateway, None).unwrap();

    if hdfs_fs.exists(&path).unwrap() {
        let mut f = hdfs::hdfs::OpenOptions::new()
            .read(true)
            .open(&hdfs_fs, path)
            .unwrap();
        let mut buffer = vec![];
        f.read_to_end(&mut buffer).unwrap();

        std::io::stdout().write_all(buffer.as_slice()).unwrap();
    } else {
        println!("File {} not found", path.display())
    }
}

const DEFAULT_PATH_STR: &str = ".hdfsrc";

fn write_config(config: &config::Config) -> Result<(), err::Error> {
    let home = dirs::home_dir();

    if home.is_none() {
        return Err(err::Error::NoHome);
    }
    let mut home = home.unwrap();
    home.push(DEFAULT_PATH_STR);

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(home)?;

    file.write_all(toml::to_string(&config)?.as_bytes())?;
    Ok(())
}

fn home_config() -> Option<config::Config> {
    let mut home = dirs::home_dir()?;
    home.push(DEFAULT_PATH_STR);

    if !home.exists() {
        return None;
    }

    let mut f = fs::File::open(home.as_os_str())
        .unwrap_or_else(|_| panic!("Error while opening {}", home.display()));

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .unwrap_or_else(|_| panic!("Error while reading {}", home.display()));

    let config: config::Config = toml::from_str(&contents).expect("Format error in config file");

    Some(config)
}

fn main() {
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    env_logger::Builder::from_env(env).init();

    let yaml = load_yaml!("cli.yml");
    let home_config = home_config();

    let matches = App::from_yaml(yaml).get_matches();
    let hadoop_install_path = env::var("HADOOP_INSTALL").map(Some).unwrap_or(None);
    let hadoop_default_gateway = env::var("GATEWAY_DEFAULT").map(Some).unwrap_or(None);
    let config = matches
        .value_of("config")
        .or_else(|| {
            home_config
                .as_ref()
                .and_then(|c| c.hadoop.as_ref())
                .and_then(|h| h.config_path.as_ref().map(String::as_ref))
        })
        .or_else(|| hadoop_install_path.as_ref().map(String::as_ref))
        .unwrap_or_else(|| {
            panic!(
                "No hadoop config path has been found. Please set hadoop config path in ~/{}",
                DEFAULT_PATH_STR
            )
        });

    let config = PathBuf::from(config);
    let gateway = matches
        .value_of("gateway")
        .or_else(|| hadoop_default_gateway.as_ref().map(String::as_ref))
        .or_else(|| {
            home_config
                .as_ref()
                .and_then(|c| c.gateway.as_ref())
                .and_then(|g| g.default.as_ref().map(String::as_ref))
        });

    if let Some(matches) = matches.subcommand_matches("ls") {
        let path = matches.value_of("PATH").unwrap();
        let path = PathBuf::from(path);
        ls(config, gateway, path);
    } else if let Some(matches) = matches.subcommand_matches("cat") {
        let path = matches.value_of("PATH").unwrap();
        let path = PathBuf::from(path);
        text(config, gateway, path);
    } else if let Some(matches) = matches.subcommand_matches("gateway") {
        if let Some(_matches) = matches.subcommand_matches("list") {
            for g in hdfs::hdfs::list_gateway(config).unwrap() {
                println!("{}", g)
            }
        } else if let Some(matches) = matches.subcommand_matches("switch") {
            let gateway = matches.value_of("switch_gateway").unwrap();

            let gateways = hdfs::hdfs::list_gateway(config).unwrap();
            if !gateways.contains(&gateway.to_owned()) {
                println!(
                    "No gateway with name \"{}\" found in hadoop config",
                    gateway
                );
                return;
            }

            let mut home_config = home_config.clone().unwrap_or_default();
            let mut gateway_config = config::Gateway::default();
            gateway_config.default = Some(gateway.to_owned());
            home_config.gateway = Some(gateway_config);
            write_config(&home_config).unwrap();
        } else if let Some(_matches) = matches.subcommand_matches("current") {
            println!("Current gateway: {}", gateway.unwrap_or("None"))
        }
    }
}
