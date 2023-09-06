#![allow(dead_code)]

use crate::clothings::{self, Clothing};
use std::{sync::Arc, path::Path};
use serde::{Serialize, Deserialize};

const INR_TO_EUR: f32 = 0.012;

#[derive(Debug, Deserialize)]
struct MyntraProductCatalog {
    #[serde(rename = "ProductName")]
    name: Arc<str>,

    #[serde(rename = "ProductBrand")]
    brand: Arc<str>,

    #[serde(rename = "Gender")]
    gender: Arc<str>,

    #[serde(rename = "Price (INR)")]
    price: f32,

    #[serde(rename = "Description")]
    description: Arc<str>,

    #[serde(rename = "PrimaryColor")]
    primary_color: Arc<str>,
}

#[derive(Debug, Serialize)]
struct ExtendedClothing {
    uuid: Arc<str>,
    name: Arc<str>,
    brand: Arc<str>,
    gender: Arc<str>,
    price: f32,
    description: Arc<str>,
    primary_color: Arc<str>,
    label: Arc<str>,
    size: clothings::Size,
    kids: bool
}

impl ExtendedClothing {

    /// Builds a extended clothing item.
    pub fn build_from_clothing_and_extension(
        clothing: Clothing,
        extension: MyntraProductCatalog
    ) -> ExtendedClothing {

        ExtendedClothing {
            uuid: clothing.uuid,
            name: extension.name,
            brand: extension.brand,
            gender: extension.gender,
            price: round_price_in_eur(convert_inr_to_eur(extension.price)),
            description: extension.description,
            primary_color: extension.primary_color,
            label: clothing.label,
            size: clothing.size,
            kids: clothing.kids,
        }
    }

    /// Extend Clothing with extra data.
    pub fn extend_clothing_csv(
        csv_clothing: Box<dyn AsRef<Path>>,
        csv_extra_data: Box<dyn AsRef<Path>>,
        destination: Box<dyn AsRef<Path>>
    ) -> Result<Box<dyn AsRef<Path>>, Box<dyn std::error::Error>> {

        let csv_clothing = csv_clothing.as_ref().as_ref();
        let csv_extra_data = csv_extra_data.as_ref().as_ref();
        let destination = destination.as_ref().as_ref();

        let mut csv_clothing_reader = csv::ReaderBuilder::new()
            .from_path(csv_clothing)?;
        let mut csv_extra_data_reader = csv::ReaderBuilder::new()
            .from_path(csv_extra_data)?;
        let mut csv_writer = csv::WriterBuilder::new()
            .from_path(destination)?;

        let clothings = csv_clothing_reader.deserialize::<Clothing>();
        let extra_data = csv_extra_data_reader.deserialize::<MyntraProductCatalog>();

        clothings
            .zip(extra_data)
            .filter_map(|(clothing, extension)|
                match clothing.is_err() || extension.is_err() {
                    true => None,
                    false => Some((clothing.unwrap(), extension.unwrap()))
                }
            )
            .map(|(clothing, extension)| {
                ExtendedClothing::build_from_clothing_and_extension(clothing, extension)
            })
            .try_for_each(|extended_clothing| {
                csv_writer.serialize(extended_clothing)
            })?;

        // for clothing in clothings {
        //     let clothing = clothing?;
        //     let extension = extra_data.next().unwrap()?;
        //     let extended_clothing = ExtendedClothing::build_from_clothing_and_extension(clothing, extension);
        //     csv_writer.serialize(extended_clothing)?;
        // }

        Ok(Box::new(destination.to_owned()))
    }

}

fn convert_inr_to_eur(price: f32) -> f32 {
    price * INR_TO_EUR
}

fn round_price_in_eur(price: f32) -> f32 {
    (price * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_convert_inr_to_eur() {
        assert_eq!(convert_inr_to_eur(100.0), 1.2);
    }

    #[test]
    fn test_round_price_in_eur() {
        assert_eq!(round_price_in_eur(1.234), 1.23);
        assert_eq!(round_price_in_eur(1.235), 1.24);
    }

    #[test]
    fn test_build_from_clothing_and_extension() {
        let clothing = Clothing {
            uuid: "1234".into(),
            label: "Brand".into(),
            size: clothings::Size::XS,
            kids: false,
        };

        let extension = MyntraProductCatalog {
            name: "T-Shirt".into(),
            brand: "Brand".into(),
            gender: "Unisex".into(),
            price: 1999.0,
            description: "A comfortable T-Shirt".into(),
            primary_color: "Blue".into(),
        };

        let expected = ExtendedClothing {
            uuid: "1234".into(),
            name: "T-Shirt".into(),
            brand: "Brand".into(),
            gender: "Unisex".into(),
            price: round_price_in_eur(convert_inr_to_eur(1999.0)),
            description: "A comfortable T-Shirt".into(),
            primary_color: "Blue".into(),
            label: "Brand".into(),
            size: clothings::Size::XS,
            kids: false,
        };

        let result = ExtendedClothing::build_from_clothing_and_extension(clothing, extension);

        assert_eq!(result.uuid, expected.uuid);
        assert_eq!(result.name, expected.name);
        assert_eq!(result.brand, expected.brand);
        assert_eq!(result.gender, expected.gender);
        assert_eq!(result.price, expected.price);
        assert_eq!(result.description, expected.description);
        assert_eq!(result.primary_color, expected.primary_color);
        assert_eq!(result.label, expected.label);
        assert_eq!(result.size, expected.size);
        assert_eq!(result.kids, expected.kids);
    }

    #[test]
    fn test_extend_clothing_csv() {
        let csv_clothing = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/resources/clothings.csv");
        let csv_extra_data = Path::new(env!("CARGO_MANIFEST_DIR")).
            join("tests/resources/myntra_products_catalog.csv");
        let destination = TempDir::new().unwrap();

        let result = ExtendedClothing::extend_clothing_csv(
            Box::new(csv_clothing),
            Box::new(csv_extra_data),
            Box::new(destination.path().join("extended_clothings.csv"))
        );

        assert!(result.is_ok());
    }

}