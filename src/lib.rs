use std::env;
use std::error::Error;
use std::path::PathBuf;

use clap::{Parser};
use glob::glob;
use serde::Serialize;
use csv;

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[arg(short, long)]
    origin: Option<PathBuf>,

    #[arg(short, long)]
    destination: Option<PathBuf>,
}

pub struct Config {
    origin: PathBuf,
    destination: PathBuf,
}

impl Config {
    pub fn build() -> Result<Config, &'static str> {

        let cli = Cli::parse();

        let current_dir: PathBuf = match env::current_dir() {
            Ok(path) => path,
            Err(_) => return Err("Could not get the current directory")
        };

        let origin = cli.origin
            .unwrap_or(current_dir.clone());

        let mut destination = cli.destination
            .unwrap_or(current_dir);

        if !destination.is_file() {
            destination.push("data.csv");
        }

        Ok(Config { origin, destination })
    }
}

impl Distribution<Size> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Size {
        match rng.gen_range(0..=4) {
            0 => Size::XS,
            1 => Size::S,
            2 => Size::M,
            3 => Size::L,
            4 => Size::XL,
            _ => panic!("Invalid size")
        }
    }
}

#[derive(Debug, Serialize)]
struct Row<'a> {
    file_name: &'a str,
    label: &'a str,
    size: Size,
    kids: bool,
}

#[derive(Debug, Serialize)]
enum Size {
    XS,
    S,
    M,
    L,
    XL
}

pub fn run (config: Config) -> Result<(), Box<dyn Error>> {
    create_csv_from_directory(config.origin, config.destination);
    Ok(())
}

fn create_csv_from_directory(origin: PathBuf, destination: PathBuf) {

    let origin = origin.into_os_string().into_string().unwrap();

    let pattern = format!("{}/**/*.jpg", origin);

    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path(&destination.as_path())
        .unwrap();

    glob(&pattern).unwrap().for_each(|path| {

        let path: PathBuf = path.unwrap();
        let row: Row = Row {
            file_name: path.
                file_name().unwrap().to_str().unwrap(),
            label: path
                .parent().unwrap()
                .components()
                .last().unwrap()
                .as_os_str().to_str().unwrap(),
            size: rand::random(),
            kids: rand::random(),
        };
        
        writer.serialize(row).unwrap();
    });
}