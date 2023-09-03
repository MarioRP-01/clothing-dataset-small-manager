mod config;
mod github_api;
mod clothings;

use config::Config;
use github_api::CompressionType;
use throbber::Throbber;

const REPOSITORY_AUTHOR: &str = "alexeygrigorev";
const REPOSITORY_NAME: &str = "clothing-dataset-small";
const REPOSITORY_BRANCH: &str = "master";
const REPOSITORY_COMPRESSION_TYPE: CompressionType = CompressionType::Tarball;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {

    let config = Config::build()?;

    let mut throbber = Throbber::new();

    let origin = match config.origin {
        Some(path) => path,
        None => {
            throbber.start_with_msg("Cloning the repository...".to_string());

            let temp_file = github_api::download_repository_into_temp(
                REPOSITORY_AUTHOR,
                REPOSITORY_NAME,
                REPOSITORY_COMPRESSION_TYPE,
                REPOSITORY_BRANCH
            ).map_err(|error| {
                    throbber.fail("Could not clone the repository".to_string());
                    error
                })?;

            throbber.success("Repository cloned successfully".to_string());

            Box::new(temp_file)
        }
    };

    throbber.start_with_msg("Extracting the dataset...".to_string());

    clothings::extract_dataset_from_path(
        origin, 
        config.destination
    ).map_err(|error| {
        throbber.fail("Could not extract the dataset".to_string());
        error
    })?;

    throbber.success("Repository extracted successfully".to_string());
    throbber.end();

    Ok(())
}
