use clap::Parser;
use git2::Repository;
use glob::glob;
use itertools::Itertools;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use serde::Serialize;
use std::{env, error::Error, fs, path::PathBuf};
use tempfile::TempDir;

const DATASET_URL: &str = "https://github.com/alexeygrigorev/clothing-dataset-small";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[arg(short, long)]
    destination: Option<PathBuf>,
}

pub struct Config {
    destination: PathBuf,
}

impl Config {
    pub fn build() -> Result<Config, &'static str> {
        let cli = Cli::parse();

        let current_dir: PathBuf = match env::current_dir() {
            Ok(path) => path,
            Err(_) => return Err("Could not get the current directory"),
        };

        let destination = cli.destination.unwrap_or(current_dir);

        if !destination.is_dir() {
            return Err("Destination must be a directory");
        }

        Ok(Config { destination })
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
    let temp_file = clone_repo_in_temp(DATASET_URL)?;
    extract_dataset(temp_file.path().to_path_buf(), config.destination)?;

    Ok(())
}

/**
 * Creates a CSV file from clothing-dataset-small directory and create a new directory for the images.
 */
fn extract_dataset(dataset: PathBuf, destination: PathBuf) -> Result<(), Box<dyn Error>> {
    // let origin = origin.into_os_string().into_string().unwrap();
    let dataset = dataset.into_os_string().into_string().unwrap();
    let pattern = format!("{}/**/*.jpg", dataset);

    let csv_file = destination.clone().join("data.csv");
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

fn clone_repo_in_temp(url: &str) -> Result<TempDir, Box<dyn std::error::Error>> {
    let temp_file = TempDir::new()?;
    Repository::clone(url, &temp_file)?;
    Ok(temp_file)
}
