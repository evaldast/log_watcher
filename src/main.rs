extern crate ansi_term;

use ansi_term::Colour;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::thread;
use std::time::Duration;

const LOG_PATH: &str = "";

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

    let file = File::open(LOG_PATH).expect("Failed opening file");
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
