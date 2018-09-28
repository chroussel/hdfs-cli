use err::HdfsErr;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

lazy_static! {
    static ref re: Regex =
        Regex::new(r"<name>(?P<key>.*)</name><value>(?P<value>.*)</value>").unwrap();
}

pub struct Config {
    config_map: HashMap<String, String>,
}

impl Config {
    pub fn new(directory: &Path) -> Result<Config, HdfsErr> {
        if !directory.exists() {
            return Err(HdfsErr::DirectoryNotFound(String::from(
                directory.to_str().unwrap_or_default(),
            )));
        }

        let default_config_file = ["core-site.xml", "hdfs-site.xml"];

        let mut configmap = HashMap::new();

        for filename in default_config_file.iter() {
            let filepath = directory.join(filename);

            let f: File = File::open(filepath)?;
            let mut file = BufReader::new(f);

            for wrapped_line in file.lines() {
                let line = wrapped_line.unwrap();
                let caps = re
                    .captures(line.as_ref())
                    .map(|v| (v.name("key"), v.name("value")));
                match caps {
                    Some((Some(key), Some(value))) => {
                        configmap.insert(key.as_str().to_owned(), value.as_str().to_owned())
                    }
                    _ => continue,
                };
            }
        }

        return Ok(Config {
            config_map: configmap,
        });
    }
}
