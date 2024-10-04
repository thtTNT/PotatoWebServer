use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::sync::OnceLock;

static CONFIG_PATH: &str = "config.json";
static CONFIG: OnceLock<Instance> = OnceLock::new();

#[derive(Deserialize, Debug)]
pub struct Instance {
    pub host: String,
    pub port: u16,
    pub home_page: String,
    pub root_dir: String,
}

impl Instance {
    pub fn global() -> &'static Instance {
        CONFIG.get_or_init(|| {
            println!("Reading config...");
            let res = read_config();
            if res.is_err() {
                panic!("Failed to read config: {}", res.err().unwrap());
            }
            println!("Config read successfully");
            res.unwrap()
        })
    }
}

pub fn read_config() -> Result<Instance, Box<dyn Error>> {
    let file = File::open(CONFIG_PATH);
    if file.is_err() {
        return Err(Box::new(file.err().unwrap()));
    }
    let reader = BufReader::new(file.unwrap());
    let json = serde_json::from_reader(reader);
    if json.is_err() {
        return Err(Box::new(json.err().unwrap()));
    }
    Ok(json.unwrap())
}
