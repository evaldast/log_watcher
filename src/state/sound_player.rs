use std::fs::File;
use std::io::{Read, Cursor};


pub struct SoundPlayer {    
    device: rodio::Device,
    file_buffer: Vec<u8>
}

impl SoundPlayer {
    pub fn new(file_location: &str) -> Self {        
        let mut file = File::open(file_location).unwrap();
        let device = rodio::default_output_device().unwrap();
        let mut file_buffer = Vec::new();

        file.read_to_end(&mut file_buffer).unwrap();

        Self {            
            device,
            file_buffer
        }
    }

    pub fn play(&self) {        
        let cursor = Cursor::new(self.file_buffer.clone());

        rodio::play_once(&self.device, cursor).unwrap().detach();
    }
}