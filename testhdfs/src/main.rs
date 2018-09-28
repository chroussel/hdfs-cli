extern crate hdfs;
use std::path::Path;

fn main() {
    let config = hdfs::conf::Config::new(Path::new(
        "/mnt/c/sources/mesos/recommendation/mesos/recosourceservice/lib/hadoop-config/hadoop",
    )).unwrap();
    let hdfs_fs = hdfs::hdfs::get_hdfs("30-e1-71-6d-f0-00.pa4.hpc.criteo.preprod").unwrap();

    match hdfs_fs.ls("/user/c.roussel") {
        Err(err) => println!("{}", err),
        Ok(value) => {
            for i in &value {
                println!("{}", i.name)
            }
        }
    };
}
