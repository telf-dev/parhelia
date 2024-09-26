// Get the image resolution/camera settings from a config file

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde::Deserialize;

use super::Vec3;


#[derive(Deserialize, Debug)]
pub struct Config { 
    pub as_w: f64,
    pub as_h: f64,
    pub im_width: u64,
    pub samples_per_pixel: u64,
    pub max_depth: u64,
    pub lookfrom: Vec3,
    pub lookat: Vec3,
    pub vup: Vec3,
    pub aperture: f64,
    pub vfov:f64,
}

impl Config{
    pub fn read_config<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn Error>> { 
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }
}