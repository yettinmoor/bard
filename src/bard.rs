use crate::block::Block;

use std::{
    collections::HashMap,
    env,
    io::Write,
    path::{Path, PathBuf},
    process::{exit, Command},
};
use yaml_rust::yaml::{Yaml, YamlLoader};

pub struct Bard {
    global: HashMap<String, String>,
    blocks: Vec<Block>,
    config_path: PathBuf,
}

impl Bard {
    pub fn init(config_path: Option<PathBuf>) -> Bard {
        // Parse config.
        let config_path = config_path.unwrap_or({
            let home = env::var("HOME").unwrap();
            let config_dir = Path::new(&home).join(".config").join("bard");
            std::fs::create_dir_all(&config_dir)
                .unwrap_or_else(|_| panic!("could not create `{}`", config_dir.to_string_lossy()));
            config_dir.join("bard.yaml")
        });
        let config = Bard::parse_config(&config_path);

        // Create global dict.
        let global = config["GLOBAL"]
            .as_hash()
            .unwrap()
            .into_iter()
            .map(|(k, v)| {
                (
                    k.as_str().unwrap().to_string(),
                    v.as_str().unwrap().to_string(),
                )
            })
            .collect::<HashMap<String, String>>();

        // Create array of block dicts.
        let blocks: Vec<Block> = {
            let hash = config.as_hash().expect("expected hash struct");
            hash.iter()
                .filter(|(name, _)| name.as_str() != Some("GLOBAL"))
                .map(|(name, block)| {
                    Block::init(
                        name.as_str().unwrap().to_string(),
                        block["cmd"].as_str().unwrap().to_string(),
                        block["prefix"].as_str().map(|s| s.to_string()),
                    )
                })
                .collect()
        };

        eprintln!(
            "bard init: [{}]",
            blocks
                .iter()
                .map(|block| block.name.clone())
                .collect::<Vec<_>>()
                .join(" ")
        );

        let mut bard = Bard {
            blocks,
            global,
            config_path,
        };
        let _ = bard.update_all();
        let _ = bard.draw_bar();
        bard
    }

    pub fn update(&mut self, blocks: &[String]) -> String {
        let mut writer = Vec::new();
        for block_name in blocks {
            let (block_name, text) = if let Some((block_name, text)) = block_name.split_once(':') {
                (block_name.to_string(), Some(text))
            } else {
                (block_name.clone(), None)
            };
            if let Some(block) = self.blocks.iter_mut().find(|s| s.name == *block_name) {
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
        eprint!("{}", s);
        s
    }

    pub fn update_all(&mut self) -> String {
        let mut writer = Vec::new();
        for block in self.blocks.iter_mut() {
            block.run(&mut writer);
        }
        let s = String::from_utf8_lossy(&writer).to_string();
        eprint!("{}", s);
        s
    }

    pub fn draw_bar(&self) -> String {
        let s = format!(
            "{}{}{}",
            self.global.get("prefix").unwrap_or(&" ".to_string()),
            self.blocks
                .iter()
                .filter_map(|s| s.draw())
                .collect::<Vec<_>>()
                .join(
                    self.global
                        .get("delim")
                        .map(|s| s.as_str())
                        .unwrap_or(" | "),
                ),
            self.global.get("suffix").unwrap_or(&" ".to_string()),
        );
        let _ = Command::new("xsetroot")
            .arg("-name")
            .arg(&s)
            .output()
            .unwrap();
        eprintln!("draw_bar: `{s}`");
        s
    }

    pub fn restart(&mut self) {
        *self = Bard::init(Some(self.config_path.clone()));
        let _ = self.update_all();
    }

    fn parse_config(config_path: &PathBuf) -> Yaml {
        let config_str = std::fs::read_to_string(&config_path).unwrap_or_else(|_| {
            panic!(
                "config file not found: `{}`",
                &config_path.to_string_lossy()
            )
        });
        match YamlLoader::load_from_str(&config_str) {
            Ok(config) => config[0].clone(),
            Err(e) => {
                eprintln!("parse error:\n{}", e);
                exit(1);
            }
        }
    }
}
