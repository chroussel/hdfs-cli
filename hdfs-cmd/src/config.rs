#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct Config {
    pub hadoop: Option<Hadoop>,
    pub gateway: Option<Gateway>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct Hadoop {
    pub config_path: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct Gateway {
    pub default: Option<String>,
}

#[cfg(test)]
mod test {

    use config::*;

    #[test]
    fn test() {
        let t = "
        [gateway]
        default = \"prod\"

        [hadoop]
        config_path = \"/home/toto/\"
        ";

        let deserialized: Config = toml::from_str(t).unwrap();
        println!("{:?}", deserialized);
    }
}
