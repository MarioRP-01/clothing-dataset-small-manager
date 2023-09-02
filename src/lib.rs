mod github_api;

use clap::Parser;
use github_api::CompressionType;
use glob::glob;
use itertools::Itertools;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use serde::Serialize;
use std::{env, error::Error, fs, path::PathBuf};
use throbber::Throbber;

const REPOSITORY_AUTHOR: &str = "alexeygrigorev";
const REPOSITORY_NAME: &str = "clothing-dataset-small";
const REPOSITORY_BRANCH: &str = "master";
const REPOSITORY_COMPRESSION_TYPE: CompressionType = CompressionType::Tarball;

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
    origin: Option<PathBuf>,
    destination: PathBuf,
}

impl Config {
    pub fn build() -> Result<Config, Box<dyn Error>> {
        let cli = Cli::parse();

        let current_dir: PathBuf = match env::current_dir() {
            Ok(path) => path,
            Err(_) => return Err("Could not get the current directory".into()),
        };

        let destination = cli.destination.unwrap_or(current_dir);

        if !destination.is_dir() {
            return Err("Destination must be an existing empty directory".into());
        }

        if destination.read_dir()?.next().is_some() {
            return Err("Destination must be an existing empty directory".into());
        }

        let origin =
            match cli.origin {
                Some(path) => match path.is_dir() {
                    true => Some(path),
                    false => return Err("Origin must be a directory".into()),
                },
                None => None
            };
        
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
            _ => panic!("Invalid size"),
        }
    }
}

#[derive(Debug, Serialize)]
struct Row {
    file_name: String,
    label: String,
    size: Size,
    kids: bool,
}

#[derive(Debug, Serialize)]
enum Size {
    XS,
    S,
    M,
    L,
    XL,
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {

    let temp_file;
    let mut throbber = Throbber::new();

    let origin = match config.origin {
        Some(path) => path,
        None => {
            throbber.start_with_msg("Cloning the repository...".to_string());

            temp_file = github_api::download_repository_into_temp(
                REPOSITORY_AUTHOR,
                REPOSITORY_NAME,
                REPOSITORY_COMPRESSION_TYPE,
                REPOSITORY_BRANCH
            ).map_err(|error| {
                    throbber.fail("Could not clone the repository".to_string());
                    error
                })?;

            throbber.success("Repository cloned successfully".to_string());

            temp_file.path().to_path_buf()
        }
    };

    throbber.start_with_msg("Extracting the dataset...".to_string());

    extract_dataset(origin, config.destination).map_err(|error| {
        throbber.fail("Could not extract the dataset".to_string());
        error
    })?;

    throbber.success("Repository extracted successfully".to_string());
    throbber.end();

    Ok(())
}

/**
 * Creates a CSV file from clothing-dataset-small directory and create a new directory for the images.
 */
fn extract_dataset(dataset: PathBuf, destination: PathBuf) -> Result<(), Box<dyn Error>> {

    let dataset = dataset.into_os_string().into_string().unwrap();
    let pattern = format!("{}/**/*.jpg", dataset);

    let csv_file = destination.join("data.csv");
    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path(csv_file.as_path())?;

    let images_dir = destination.join("images");

    fs::create_dir_all(&images_dir).unwrap();

    glob(&pattern)
        .unwrap()
        .filter_map(|path| path.ok())
        .unique_by(|path| path.file_name().unwrap().to_str().unwrap().to_owned())
        .inspect(|path| {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            fs::copy(path, images_dir.clone().join(file_name)).unwrap();
        })
        .map(|path| Row {
            file_name: path.file_name().unwrap().to_str().unwrap().to_owned(),
            label: path
                .parent()
                .unwrap()
                .components()
                .last()
                .unwrap()
                .as_os_str()
                .to_str()
                .unwrap()
                .to_owned(),
            size: rand::random(),
            kids: rand::random(),
        })
        .for_each(|row| {
            writer.serialize(row).unwrap();
        });

        Ok(())
}
