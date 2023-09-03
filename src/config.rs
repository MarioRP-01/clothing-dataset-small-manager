use clap::Parser;
use std::{env, error::Error, path::{PathBuf, Path}};

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
    pub(crate) origin: Option<Box<dyn AsRef<Path>>>,
    pub(crate) destination: Box<dyn AsRef<Path>>,
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

        let destination = Box::new(destination);

        let origin =
            match cli.origin {
                Some(path) => match path.is_dir() {
                    true => Some(Box::new(path) as Box<dyn AsRef<Path>>),
                    false => return Err("Origin must be a directory".into()),
                },
                None => None
            };

        Ok(Config { origin, destination })
    }
}
