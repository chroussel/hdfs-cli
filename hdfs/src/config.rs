use err::Error;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
pub struct Config {
    pub config_map: HashMap<String, String>,
}

impl Config {
    fn read_config_files(config_files: &[PathBuf]) -> Result<Config, Error> {
        let mut configmap = HashMap::new();

        for filepath in config_files.iter() {
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
        Ok(Config {
            config_map: configmap,
        })
    }

    pub fn new<P: AsRef<Path>>(directory: P) -> Result<Config, Error> {
        let path = directory.as_ref().to_owned();
        if !path.exists() {
            return Err(Error::DirectoryNotFound(path));
        }

        let default_config_file = ["core-site.xml", "hdfs-site.xml"];
        let config_files: Vec<PathBuf> = default_config_file
            .iter()
            .map(|filename| path.clone().join(filename))
            .collect();

        Config::read_config_files(config_files.as_slice())
    }

    fn get_host_port(configmap: &HashMap<String, String>) -> (Option<String>, Option<String>) {
        if let Some(value) = configmap.get("fs.defaultFS") {
            if value.starts_with("hdfs://") {
                let (_, text) = value.split_at(7);
                let sp: Vec<&str> = text.splitn(2, ':').collect();
                let host = sp[0];
                let port = sp[1];
                return (Some(host.to_owned()), Some(port.to_owned()));
            }
        }

        if let Some(value) = configmap.get("dfs.namenode.rpc-address") {
            let (_, text) = value.split_at(7);
            let sp: Vec<&str> = text.splitn(2, ':').collect();
            let host = sp[0];
            let port = sp[1];
            return (Some(host.to_owned()), Some(port.to_owned()));
        }

        if let Some(value) = configmap.get("dfs.nameservices") {
            let sp: Vec<&str> = value.splitn(2, ',').collect();
            let host = sp[0];
            return (Some(host.to_owned()), None);
        }
        (None, None)
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.config_map.get(key).map(|s| s.as_str())
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

#[cfg(test)]
mod test {
    use config::Config;
    use fs_test;
    use std::collections::HashMap;
    use std::env;
    use std::fs;
    use std::io::prelude::*;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_parse_xml_config() {
        let xml_string = "
        <root>
            <property>
            <name>name</name><value>value</value>
             </property>
        </root>
        ";
        let dir = fs_test::TempDir::default();
        let dir_path = dir.path();

        let file_path = dir_path.clone().join("a");
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(xml_string.as_bytes());
        drop(file);
        let config = Config::read_config_files(&[file_path]).unwrap();

        let mut hashmap = HashMap::new();
        hashmap.insert(String::from("name"), String::from("value"));
        let expected = Config {
            config_map: hashmap,
        };

        assert_eq!(expected, config)
    }
}
