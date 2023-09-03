mod github_api;
mod clothings;

use clap::Parser;
use github_api::CompressionType;
use std::{env, error::Error, path::PathBuf};
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

    clothings::extract_dataset_from_path(
        Box::new(origin), 
        Box::new(config.destination)
    ).map_err(|error| {
        throbber.fail("Could not extract the dataset".to_string());
        error
    })?;

    throbber.success("Repository extracted successfully".to_string());
    throbber.end();

    Ok(())
}
