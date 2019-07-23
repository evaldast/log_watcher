extern crate ansi_term;
extern crate toml;

use ansi_term::Colour;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::thread;
use std::time::Duration;
use toml::Value;

const CONFIG_FILE_NAME: &str = "config.toml";
const CONFIG_LOG_PATH_PROPERTY: &str = "log_path";

enum LogEvent {
    Error,
    Debug,
    Info,
    Unknown,
}

fn main() {
    let mut error_messages: Vec<String> = vec![];
    let mut info_messages: Vec<String> = vec![];
    let mut debug_messages: Vec<String> = vec![];

    let config = std::fs::read_to_string(CONFIG_FILE_NAME)
        .expect("Failed loading config file")
        .parse::<Value>()
        .expect("Failed loading config values");

    let log_path = config[CONFIG_LOG_PATH_PROPERTY]
        .as_str()
        .expect("Failed loading config value log_path");

    let file = File::open(log_path)
        .expect("Failed opening file");
        
    let mut reader = BufReader::new(file);
    let mut buffer = String::new();

    loop {
        thread::sleep(Duration::from_secs(1));

        loop {
            buffer.truncate(0);
            reader
                .read_line(&mut buffer)
                .expect("Failed reading file buffer");

            if buffer.is_empty() {
                break;
            }

            match get_event_type(&buffer) {
                LogEvent::Debug => {
                    println!("{}", Colour::Yellow.bold().paint(&buffer));
                    debug_messages.push(buffer.clone());
                }
                LogEvent::Info => {
                    println!("{}", Colour::Blue.bold().paint(&buffer));
                    info_messages.push(buffer.clone());
                }
                LogEvent::Error => {
                    println!("{}", Colour::Red.bold().paint(&buffer));
                    error_messages.push(buffer.clone());
                }
                LogEvent::Unknown => continue,
            }
        }
    }
}

fn get_event_type(event: &str) -> LogEvent {
    if event.contains("[DBG]") {
        return LogEvent::Debug;
    }

    if event.contains("[INF]") {
        return LogEvent::Info;
    }

    if event.contains("[ERR]") {
        return LogEvent::Error;
    }

    LogEvent::Unknown
}
