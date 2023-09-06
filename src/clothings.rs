use glob::glob;
use std::{path::Path, fs, sync::{Arc, Mutex}};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use rayon::prelude::*;
use serde::{Serialize, Deserialize};

/// Clothing item.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Clothing {
    pub uuid: Arc<str>,
    pub label: Arc<str>,
    pub size: Size,
    pub kids: bool,
}

impl Clothing {
    pub fn build_from_path(path: Box<dyn AsRef<Path>>) -> Result<Clothing, Box<dyn std::error::Error>> {
        
        let path = path.as_ref().as_ref();

        let file_name = path.file_stem().ok_or("File name not found")?
            .to_owned().into_string().unwrap_or("Cannot convert file name to string".to_owned());
        
        let label = path.parent().ok_or("Parent directory not found")?
            .components().last().ok_or("Cannot get parent directory")?
            .as_os_str().to_owned()
            .into_string().unwrap_or("Cannot convert parent directory to string".to_owned());

        Ok(Clothing {
            uuid: file_name.into(),
            label: label.into(),
            size: Size::random(),
            kids: rand::random(),
        })
    }
}

/// Clothing size.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(crate) enum Size {
    XS,
    S,
    M,
    L,
    XL,
}

impl Size {
    pub(crate) fn random() -> Self {
        rand::random()
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

/// Given the clothing-dataset-small directory, extract the dataset into the destination 
/// creating a csv with the directory information.
pub fn extract_dataset_from_path(
    dataset: Box<dyn AsRef<Path>>,  
    destination: Box<dyn AsRef<Path>>
) -> Result<(), Box<dyn std::error::Error>> {

    let dataset = dataset.as_ref().as_ref();
    let destination = destination.as_ref().as_ref();

    let pattern = format!("{}/**/*.jpg", dataset.to_str().unwrap());

    let csv_file = destination.join("data.csv");
    let csv_writer = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path(csv_file)?;
    let csv_writer = Arc::new(Mutex::new(csv_writer));

    let images_dir = destination.join("images");
    fs::create_dir_all(&images_dir)?;

    glob(&pattern)?
        .par_bridge()
        .filter_map(|path| path.ok())
        .inspect(|path| {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            fs::copy(path, images_dir.clone().join(file_name)).unwrap();
        })
        .filter_map(|path|
            Clothing::build_from_path(Box::new(path)).ok()
        )
        .for_each(|row| {
            csv_writer.lock().unwrap().serialize(row).unwrap();
        });

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, ops::Deref};
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_build_from_path() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("clothing-dataset-small")
            .join("jeans")
            .join("00000000.jpg");

        let clothing = Clothing::build_from_path(Box::new(path)).unwrap();

        assert_eq!(clothing.label.deref(), "jeans");
        assert_eq!(clothing.uuid.deref(), "00000000");
    }

    #[test]
    fn test_extract_dataset_from_path() {

        let dataset = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("clothing-dataset-small");
        let destination = TempDir::new().unwrap(); 

        extract_dataset_from_path(
            Box::new(dataset), 
            Box::new(destination)
        ).unwrap();
    }
}
