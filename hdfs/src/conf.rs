use err::HdfsErr;
use quick_xml::events::Event;
use quick_xml::Reader;
use regex::Regex;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

lazy_static! {
    static ref re: Regex =
        Regex::new(r"(?m)<name>(?P<key>(?s:.)*)</name><value>(?P<value>(?s:.)*)</value>").unwrap();
}

pub struct Config {
    pub config_map: HashMap<String, String>,
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
            let mut reader = Reader::from_reader(file);
            reader.trim_text(true);
            let mut buf = Vec::new();
            let mut is_key = false;
            let mut key: Option<String> = None;
            let mut value: Option<String> = None;
            let mut ignore = false;
            loop {
                match reader.read_event(&mut buf) {
                    Ok(Event::Start(ref e)) => match e.name() {
                        b"name" => {
                            is_key = true;
                            ignore = false;
                        }
                        b"value" => {
                            is_key = false;
                            ignore = false;
                        }
                        b"final" => ignore = true,
                        _ => {}
                    },
                    Ok(Event::Text(e)) => {
                        if !ignore {
                            if is_key {
                                key = Some(e.unescape_and_decode(&reader).unwrap());
                            } else {
                                value = Some(e.unescape_and_decode(&reader).unwrap());
                            }
                        }
                    }
                    Ok(Event::End(ref e)) if e.name() == b"property" => {
                        debug!("end event: property ({:?}, {:?})", key, value);
                        if let (Some(k), Some(v)) = (key, value) {
                            configmap.insert(k, v);
                        }
                        key = None;
                        value = None;
                        ignore = false;
                    }
                    Ok(Event::Eof) => break,
                    Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                    _ => {}
                }
            }
        }

        let (host, port) = Config::get_host_port(&configmap);
        if let Some(host) = host {
            configmap.insert(String::from("host"), host);
        }

        if let Some(port) = port {
            configmap.insert(String::from("port"), port);
        }
        return Ok(Config {
            config_map: configmap,
        });
    }

    fn get_host_port(configmap: &HashMap<String, String>) -> (Option<String>, Option<String>) {
        if let Some(value) = configmap.get("fs.defaultFS") {
            if value.starts_with("hdfs://") {
                let (_, text) = value.split_at(7);
                let sp: Vec<&str> = text.splitn(2, ":").collect();
                let host = sp[0];
                let port = sp[1];
                return (Some(host.to_owned()), Some(port.to_owned()));
            }
        }

        if let Some(value) = configmap.get("dfs.namenode.rpc-address") {
            let (_, text) = value.split_at(7);
            let sp: Vec<&str> = text.splitn(2, ":").collect();
            let host = sp[0];
            let port = sp[1];
            return (Some(host.to_owned()), Some(port.to_owned()));
        }

        if let Some(value) = configmap.get("dfs.nameservices") {
            let sp: Vec<&str> = value.splitn(2, ",").collect();
            let host = sp[0];
            return (Some(host.to_owned()), None);
        }
        (None, None)
    }

    pub fn get_string(&self, key: &str) -> Option<&std::string::String> {
        self.config_map.get(key)
    }

    pub fn get_all_key_values(&self) -> Vec<(String, String)> {
        self.config_map
            .iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect()
    }

    pub fn get<T>(&self, key: &str) -> Option<T>
    where
        T: std::str::FromStr,
    {
        if let Some(int_string) = self.config_map.get(key) {
            if let Ok(value) = int_string.parse::<T>() {
                Some(value)
            } else {
                None
            }
        } else {
            None
        }
    }
}
