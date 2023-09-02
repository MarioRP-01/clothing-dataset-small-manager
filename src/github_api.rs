#![allow(dead_code)]

use std::{fs::File, io::Write, path::Path};
use reqwest::{Url, header};
use tempfile::TempDir;
use flate2::read::GzDecoder;
use tar::Archive;


const GITHUB_API_URL: &str = "https://api.github.com";


pub enum CompressionType {
    Tarball,
    Zipball
}

impl ToString for CompressionType {
    fn to_string(&self) -> String {
        match self {
            CompressionType::Tarball => "tarball".to_owned(),
            CompressionType::Zipball => "zipball".to_owned()
        }
    }
}

struct GitHubBlockingClient {
    client: reqwest::blocking::Client
}

pub fn build_github_blocking_client() -> Result<reqwest::blocking::Client, Box<dyn std::error::Error>> {

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "X-GitHub-Api-Version", 
        header::HeaderValue::from_static("2022-11-28")
    );

    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
        ))
        .build()?;

    Ok(client)
}

/// Builds the download URL for a GitHub repository.
fn build_download_url(
    author: &str,
    repository: &str,
    compression_type: CompressionType,
    branch: &str
) -> Url {

    let mut url = Url::parse(GITHUB_API_URL).unwrap();
    url.path_segments_mut()
        .unwrap()
        .push("repos")
        .push(author)
        .push(repository)
        .push(&compression_type.to_string())
        .push(branch);
    url
} 

/// Downloads a repository from GitHub using GitHub API.
/// 
/// # Arguments
/// 
/// * `repository_url` - The URL of the repository to download.
/// * `compression_type` - The compression type to use.
/// 
pub fn download_repository_into_temp(
    author: &str,
    repository: &str,
    compression_type: CompressionType,
    branch: &str
) -> Result<TempDir, Box<dyn std::error::Error>> {

    let url = build_download_url(author, repository, compression_type, branch);
    let client = build_github_blocking_client()?;

    let response = client.get(url).send()?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to download repository: {}", response.status()).into());
    }

    let temp_file = TempDir::new()?;

    // Can't use File::options because the artifact don't work with GzDecoder.

    File::create(temp_file.path().join("repo.tar.gz"))?
        .write_all(&response.bytes()?)?;

    let tar_gz = File::open(temp_file.path().join("repo.tar.gz"))?;

    decompress_tar_gz(&tar_gz, &temp_file.path().join("repo"))?;

    Ok(temp_file)
}

/// Decompresses a tar.gz file into a directory.
fn decompress_tar_gz(
    tar_gz: & File,
    output_dir: &Path
) -> Result<(), Box<dyn std::error::Error>> {

    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(output_dir)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, fs};

    use super::*;

    #[test]
    fn test_build_download_url() {
        let url = build_download_url(
            "alexeygrigorev",
            "clothing-dataset-small",
            CompressionType::Tarball,
            "main"
        );

        assert_eq!(
            url.as_str(),
            "https://api.github.com/repos/alexeygrigorev/clothing-dataset-small/tarball/main"
        );

    }

    #[test]
    fn test_download_repository_into_temp() {
        let temp_dir = download_repository_into_temp(
            "rtyley",
            "small-test-repo",
            CompressionType::Tarball,
            "master"
        ).unwrap();

        assert!(temp_dir.path().exists());
    }

    #[test]
    fn test_decompress_tar_gz() {

        let tar_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("resources")
            .join("example.tar.gz");

        let tempdir = TempDir::new().unwrap();

        let tar_gz = File::open(tar_path).unwrap();
        let output_dir = tempdir.path().join("example");

        decompress_tar_gz(&tar_gz, &output_dir).unwrap();

        assert_ne!(fs::read_dir(output_dir).unwrap().count(), 0);
    }
}

