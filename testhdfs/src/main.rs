#[macro_use]
extern crate log;
extern crate env_logger;
extern crate hdfs;
use std::path::Path;

fn main() {
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    env_logger::Builder::from_env(env).init();

    let path = Path::new(
        "/Users/c.roussel/mesos/recommendation/mesos/recosourceservice/lib/hadoop-config/hadoop",
    );
    let host = String::from("preprod-pa4");
    let user = String::from("c.roussel");

    info!("Starting Connect");
    let hdfs_fs_r = hdfs::hdfs::get_hdfs(path, Some(&host), None);

    let hdfs_fs = hdfs_fs_r.unwrap();

    info!("Connect Success");
    let listing_path = "/user/c.roussel";
    info!("Listing: {}", listing_path);
    let value = hdfs_fs.ls(listing_path).unwrap();
    for i in &value {
        println!("{}", i.name)
    }
}
