use serde::{Deserialize};
use std::io::{Read};
use std::collections::HashMap;

pub async fn read<T: Read>(reader: T) -> Result<Config, serde_yaml::Error>{
    tokio::task::block_in_place(move ||{
        serde_yaml::from_reader(reader)
    })
}

#[derive(Debug, Deserialize)]
pub struct Config{
    pub creds: HashMap<String, ConfigDockerCreds>,
    pub map_copy: ConfigMapCopy,
    pub genmap: ConfigGenMap,
    pub tiler: ConfigTiler,
    pub upload: ConfigUpload,
}

#[derive(Debug, Deserialize)]
#[serde(tag="type")]
pub enum ConfigDockerCreds{
    #[serde(rename="token")]
    Token {
        token: String
    },
    #[serde(rename="gce")]
    Gce,
}

#[derive(Debug, Deserialize)]
pub struct ConfigMapCopy{
    pub docker: ConfigDocker,
    pub args: Vec<String>,
    // pub rsync_args: ConfigRsyncArgs,
}

pub trait StrVec {
    fn to_strvec(&self) -> Vec<&str>;
}

impl<T> StrVec for Vec<T>
    where T: AsRef<str> {
        
    fn to_strvec(&self) -> Vec<&str> {
        self.iter().map(AsRef::<str>::as_ref).collect()
    }
}

#[derive(Debug, Deserialize)]
pub struct ConfigDocker {
    pub image: String,
    pub cred: Option<String>,
    pub volumes: Vec<String>
}

// impl ConfigDocker {
//     pub fn volumes_strvec(&self) -> Vec<&str> {
//         self.volumes.iter().map(|s| s as &str).collect()
//     }
// }

#[derive(Debug, Deserialize)]
pub struct ConfigRsyncArgs {
    pub port: u16,
    pub key: String,
    pub copy_from: String,
    pub copy_to: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfigGenMap {
    pub docker: ConfigDocker,
    pub args: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigTiler {
    pub docker: ConfigDocker,
    pub args: Vec<String>,
    // pub args: ConfigTilerArgs,
}

// #[derive(Debug, Deserialize)]
// pub struct ConfigTilerArgs {
//     pub output: String,
//     pub max: u16,
// }

#[derive(Debug, Deserialize)]
pub struct ConfigUpload {
    pub docker: ConfigDocker,
    pub args: Vec<String>,
}