#[macro_use]
extern crate log;
extern crate env_logger;
extern crate hdfs;
use std::path::Path;
use std::io::BufRead;
use std::io::BufReader;
use std::io::prelude::*;

fn main() {
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    env_logger::Builder::from_env(env).init();

    let path = Path::new(
        "/Users/c.roussel/mesos/recommendation/mesos/recosourceservice/lib/hadoop-config/hadoop",
    );
    let host = String::from("prod-pa4");

    info!("Starting Connect");
    let hdfs_fs = hdfs::hdfs::get_hdfs(path, Some(&host), None).unwrap();

    info!("Connect Success");
    let listing_path = "/user/recocomputer/bestofs/";
    info!("Listing: {}", listing_path);
    let value = hdfs_fs.ls(listing_path).unwrap();
    for i in &value {
        println!("{}", i.name)
    }

    let fileLocation = Path::new("/user/c.roussel/toto");
    info!("Reading file: {}", fileLocation.to_str().unwrap());

    let f = hdfs::hdfs::OpenOptions::new()
        .read(true)
        .open(&hdfs_fs,fileLocation).unwrap();

    let mut file = BufReader::new(f);
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    info!("Content: {}", contents);

    let fileLocation2 = Path::new("/user/c.roussel/toto2");
    info!("Writing in file: {}", fileLocation2.to_str().unwrap());

    /*let mut f = hdfs::hdfs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(&hdfs_fs, fileLocation2).unwrap();

    let data = "Hello again\n";
    f.write_all(data.as_bytes()).unwrap();
    f.flush().unwrap();*/
    let r = hdfs_fs.exists("/user/c.roussel/toto3");
    info!("{:?}", r);
    let r = hdfs_fs.exists(fileLocation2.to_str().unwrap());
    info!("{:?}", r);


}
