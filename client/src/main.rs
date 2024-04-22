mod routing;
mod tun;
#[macro_use]
mod errors;
mod consts;

use std::{path::Path, net::IpAddr, collections::{HashMap, HashSet}};

use clap::Parser;
use errors::*;
use consts::*;
use log::info;
use serde_derive::{Deserialize, Serialize};

// TODO: Custom close-out error handling

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cconf: CliArgs = CliArgs::parse();
    let fconf: FileConfig = toml::from_str(&std::fs::read_to_string(get_config_path(&cconf)?)?)?;
    let conf = Config::merge(&fconf, &cconf).await?;
    colog::default_builder().filter_level(if conf.verbose {log::LevelFilter::Trace} else {log::LevelFilter::Error}).init();
    info!("ACC initialized and configuration loaded. Verbose mode enabled.");
    // Next step: daemonization
    Ok(())
}

struct Config {
    version: usize,
    server: usize,
    interface: u32,
    features: HashSet<Features>,
    verbose: bool,
    servers: Vec<Server>,
    daemon: bool,
}

impl Config {
    pub async fn merge(f: &FileConfig, c: &CliArgs) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            version: if let Some(a) = c.protocol {a} else if let Some(b) = f.general.version {b} else {ACC_DEFAULT_VERSION},
            server: if let Some(d) = c.server {d} else if let Some(e) = f.general.server {e} else {0},
            interface: if let Some(ref h) = c.interface.clone() {
                if let Some(z) = ifname_to_ifindex(h) {z} else {enonei!(h)?}
            } else if let Some(ref i) = f.general.interface.clone() {
                if let Some(z) = ifname_to_ifindex(i) {z} else {enonei!(i)?}
            } else {
                let handle = net_route::Handle::new()?;
                let routes = handle.list().await?;
                let mut defaults: HashMap<u32, usize> = HashMap::new();
                for route in routes {
                    if route.prefix == 0 {
                        if let Some(i) = route.ifindex {
                            defaults.entry(i).and_modify(|v| *v += 1).or_insert(1);
                        }
                    }
                }
                if defaults.len() == 0 {
                    enodei!()?;
                }
                let mut max_if = 0;
                let mut max_val = 0;
                for (k, v) in defaults.iter() {
                    if *v > max_val {
                        max_if = *k;
                        max_val = *v;
                    }
                }
                max_if
            },
            features: {
                let mut res: HashSet<Features> = HashSet::new();
                if let Some(ref feats) = f.general.features {
                    for feat in feats {
                        res.insert(Features::try_from_str(&feat)?);
                    }
                }
                res
            },
            verbose: if let Some(k) = f.general.verbose {k} else {c.verbose},
            servers: {
                let mut res: Vec<Server> = vec![];
                for srv in &f.servers {
                    res.push(srv.clone().try_into()?);
                }
                res
            },
            daemon: if let Some(h) = f.general.daemon {h} else {!c.nodaemon}
        })
    }
}

fn ifname_to_ifindex(name: &str) -> Option<u32> {
    let interfaces = netdev::interface::get_interfaces();
    for i in interfaces {
        if name == i.name {
            return Some(i.index);
        }
    }
    None
}

#[derive(Eq, PartialEq, Hash)]
enum Features {
    GfwPrefix,
    GfwEntropy,
    GfwSlowAscii,
    GfwAscii,
}

impl Features {
    pub fn try_from_str(inp: &str) -> Result<Self, Box<dyn std::error::Error>>  {
        Ok(match inp {
            "GfwPrefix" => Self::GfwPrefix,
            "GfwEntropy" => Self::GfwEntropy,
            "GfwSlowAscii" => Self::GfwSlowAscii,
            "GfwAscii" => Self::GfwAscii,
            _ => ebargs!(inp)?
        })
    }
}

struct Server {
    pub low: u16,
    pub high: u16,
    pub proto: MasqueradeProto,
    pub ip: IpAddr
}

impl TryFrom<FileConfigServer> for Server {
    type Error = ArgParseError;
    fn try_from(value: FileConfigServer) -> Result<Self, Self::Error> {
        Ok(Self {
            low: value.lowport,
            high: value.highport,
            proto: MasqueradeProto::try_from_str(&value.proto)?,
            ip: if let Ok(v) = value.ip.parse() {v} else {ebargsnb!(value.ip)?}
        })
    }
}

enum MasqueradeProto {
    Tcp,
    Udp,
    Http,
    Https,
    Quic,
}

impl MasqueradeProto {
    pub fn try_from_str(inp: &str) -> Result<Self, ArgParseError> {
        Ok(match inp {
            "tcp" => Self::Tcp,
            "udp" => Self::Udp,
            "http" => Self::Http,
            "https" => Self::Https,
            "quic" => Self::Quic,
            _ => ebargsnb!(inp)?
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct FileConfig {
    #[serde(rename = "General")]
    pub general: FileConfigGeneral,
    #[serde(rename = "Server")]
    pub servers: Vec<FileConfigServer>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FileConfigGeneral {
    pub version: Option<usize>,
    pub server: Option<usize>,
    pub interface: Option<String>,
    pub features: Option<Vec<String>>,
    pub verbose: Option<bool>,
    pub daemon: Option<bool>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FileConfigServer {
    pub lowport: u16,
    pub highport: u16,
    // NOTE: proto should pull from a list (maybe even a hashmap) of middlewares
    pub proto: String,
    pub ip: String,
}

fn get_config_path(args: &CliArgs) -> Result<String, Box<dyn std::error::Error>> {
    let shlex = shellexpand::tilde(if cfg!(windows) {
        "~/AppData/Roaming/ACC/acc.toml"
    } else {
        "~/.config/acc.toml"
    })
    .to_string();
    if let Some(ref z) = args.config {
        if Path::new(z).exists() {
            Ok(z.to_string())
        } else {
            enocnf!()
        }
    } else if let Ok(v) = std::env::var("ACC_CONFIG") {
        if Path::new(&v).exists() {
            Ok(v.to_string())
        } else {
            enocnf!()
        }
    } else if Path::new(&shlex).exists() {
        Ok(shlex)
    } else if cfg!(unix) && Path::new("/etc/acc.toml").exists() {
        Ok("/etc/acc.toml".to_string())
    } else if cfg!(windows) && Path::new("C:/Program Files/ACC/acc.toml").exists() {
        Ok("C:/Program Files/Common Files/acc.toml".to_string())
    } else if cfg!(debug_assertions) && Path::new("./sample-config.toml").exists() {
        Ok("./sample-config.toml".to_string())
    } else {
        enocnf!()
    }
}

#[derive(clap::Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct CliArgs {
    #[clap(long)]
    pub protocol: Option<usize>,
    #[clap(long, short)]
    pub server: Option<usize>,
    #[clap(long)]
    pub config: Option<String>,
    #[clap(long, short, action)]
    pub verbose: bool,
    #[clap(long)]
    pub interface: Option<String>,
    // features (split automatically between features and features_danger which is sugar)
    #[clap(long)]
    pub features: Option<String>,
    #[clap(long, action)]
    pub nodaemon: bool,
}
