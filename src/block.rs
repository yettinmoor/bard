use std::io::Write;
use std::ops::Not;
use std::process::{Command, Stdio};

pub struct Block {
    pub name: String,
    cmd: String,
    prefix: Option<String>,
    last_output: String,
}

impl Block {
    pub fn init(name: String, cmd: String, prefix: Option<String>) -> Block {
        Block {
            name,
            cmd,
            prefix,
            last_output: String::new(),
        }
    }
    pub fn run(&mut self, writer: &mut dyn Write) {
        let output = Command::new("sh")
            .arg("-c")
            .arg(&self.cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .unwrap();
        self.last_output = String::from_utf8_lossy(&output.stdout)
            .replace('\n', " ")
            .trim()
            .to_string();
        write!(writer, "[{}]: {}", self.name, output.status).unwrap();
        if !output.status.success() && !output.stderr.is_empty() {
            write!(
                writer,
                ", stderr: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )
            .unwrap();
        }
        writeln!(writer).unwrap();
    }

    pub fn set_output(&mut self, text: &str, writer: &mut dyn Write) {
        self.last_output = text.to_string();
        writeln!(writer, "[{}]: set to `{}`", self.name, text).unwrap();
    }

    pub fn draw(&self) -> Option<String> {
        self.last_output.is_empty().not().then(|| {
            format!(
                "{}{}",
                self.prefix
                    .as_ref()
                    .map(|p| format!("{} ", p))
                    .unwrap_or_else(|| "".to_string()),
                self.last_output
            )
        })
    }
}
