use crate::block::Block;

use log::{error, info};
use std::{
    env,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};
use yaml_rust::{yaml::YamlLoader, ScanError};

mod defaults {
    pub const DELIM: &str = " | ";
    pub const PREFIX: &str = " ";
    pub const SUFFIX: &str = " ";
}

pub struct Bard {
    status: Status,
    config_path: PathBuf,
}

enum Status {
    ParseOk {
        delim: String,
        prefix: String,
        suffix: String,
        blocks: Vec<Block>,
    },
    ParseError(ParseError),
}

impl Bard {
    pub fn init(config_path: Option<PathBuf>) -> Bard {
        let config_path = config_path.unwrap_or({
            let home = env::var("HOME").unwrap();
            let config_dir = Path::new(&home).join(".config").join("bard");
            std::fs::create_dir_all(&config_dir)
                .unwrap_or_else(|_| panic!("could not create `{}`", config_dir.to_string_lossy()));
            config_dir.join("bard.yaml")
        });
        let bard = Bard::parse_config(config_path);
        if let Status::ParseError(err) = &bard.status {
            err.log();
        }
        bard
    }

    pub fn update(&mut self, block_names: &[String]) -> (bool, String) {
        match &mut self.status {
            Status::ParseOk { blocks, .. } => {
                let mut writer = Vec::new();
                for block_name in block_names {
                    let (block_name, text) =
                        if let Some((block_name, text)) = block_name.split_once(':') {
                            (block_name.to_string(), Some(text))
                        } else {
                            (block_name.clone(), None)
                        };
                    if let Some(block) = blocks.iter_mut().find(|s| s.name == *block_name) {
                        if let Some(text) = text {
                            block.set_output(text, &mut writer);
                        } else {
                            block.run(&mut writer);
                        }
                    } else {
                        writeln!(writer, "[{}]: block not found", block_name).unwrap();
                    }
                }
                let s = String::from_utf8_lossy(&writer).to_string();
                info!("{}", s.trim_end());
                (true, s)
            }
            Status::ParseError(err) => (false, err.log()),
        }
    }

    pub fn update_all(&mut self) -> (bool, String) {
        match &mut self.status {
            Status::ParseOk { blocks, .. } => {
                let mut writer = Vec::new();
                for block in blocks.iter_mut() {
                    block.run(&mut writer);
                }
                let s = String::from_utf8_lossy(&writer).to_string();
                info!("{}", s.trim_end());
                (true, s)
            }
            Status::ParseError(err) => (false, err.log()),
        }
    }

    pub fn draw_bar(&self) -> String {
        if let Status::ParseOk {
            blocks,
            delim,
            prefix,
            suffix,
        } = &self.status
        {
            let s = format!(
                "{}{}{}",
                prefix,
                blocks
                    .iter()
                    .filter_map(|s| s.draw())
                    .collect::<Vec<_>>()
                    .join(delim.as_str()),
                suffix,
            );
            let _ = Command::new("xsetroot")
                .arg("-name")
                .arg(&s)
                .output()
                .unwrap();
            info!("draw_bar: `{s}`");
            s
        } else {
            unreachable!()
        }
    }

    pub fn restart(&mut self) {
        *self = Bard::init(Some(self.config_path.clone()));
    }

    fn parse_config(config_path: PathBuf) -> Bard {
        let config_str = std::fs::read_to_string(&config_path).unwrap_or_else(|_| {
            panic!(
                "config file not found: `{}`",
                &config_path.to_string_lossy()
            )
        });

        let res = YamlLoader::load_from_str(&config_str)
            .map_err(ParseError::Scan)
            .and_then(|config| {
                if config.is_empty() {
                    Err(ParseError::BadStruct)
                } else {
                    Ok(config[0].clone())
                }
            });

        match res {
            Ok(config) => {
                let delim = config["delim"]
                    .as_str()
                    .unwrap_or(defaults::DELIM)
                    .to_string();
                let prefix = config["prefix"]
                    .as_str()
                    .unwrap_or(defaults::PREFIX)
                    .to_string();
                let suffix = config["suffix"]
                    .as_str()
                    .unwrap_or(defaults::SUFFIX)
                    .to_string();

                let blocks: Vec<Block> = {
                    let mut blocks = vec![];
                    let hash = if let Some(hash) = config["blocks"].as_hash() {
                        hash
                    } else {
                        error!("expected `blocks` hash");
                        return Bard {
                            status: Status::ParseError(ParseError::BadStruct),
                            config_path,
                        };
                    };
                    for (name, block) in hash.into_iter() {
                        let name = name.as_str().unwrap().to_string();
                        let cmd = if let Some(cmd) = block["cmd"].as_str() {
                            cmd.to_string()
                        } else {
                            return Bard {
                                status: Status::ParseError(ParseError::MissingField {
                                    block: name,
                                    field: "cmd".to_string(),
                                }),
                                config_path,
                            };
                        };
                        blocks.push(Block::init(
                            name,
                            cmd,
                            block["prefix"].as_str().map(|s| s.to_string()),
                        ));
                    }
                    blocks
                };

                info!(
                    "bard init: [{}]",
                    blocks
                        .iter()
                        .map(|block| block.name.clone())
                        .collect::<Vec<_>>()
                        .join(" ")
                );

                let mut bard = Bard {
                    status: Status::ParseOk {
                        blocks,
                        delim,
                        prefix,
                        suffix,
                    },
                    config_path,
                };
                let _ = bard.update_all();
                let _ = bard.draw_bar();
                bard
            }
            Err(err) => Bard {
                status: Status::ParseError(err),
                config_path,
            },
        }
    }
}

enum ParseError {
    Scan(ScanError),
    MissingField { block: String, field: String },
    BadStruct,
}

impl ParseError {
    fn log(&self) -> String {
        let reply = match self {
            ParseError::Scan(err) => format!("parse error: {}", err),
            ParseError::MissingField { block, field } => {
                format!("parse error: expected `{}` field in [{}]", field, block)
            }
            ParseError::BadStruct => "parse error: config is not a proper yaml hash".to_string(),
        };
        error!("{}", reply);
        reply + "\n"
    }
}
