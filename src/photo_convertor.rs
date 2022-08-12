use std::fs::File;

use bytes::Bytes;
use tracing::error;

use crate::api::ij;
use crate::api_error::ApiError;
use crate::helpers::gen_random_hex;
use crate::parse_env::AppEnv;

#[derive(Debug, Clone)]
pub struct PhotoEnv {
    location_original: String,
    location_converted: String,
    location_watermark: String,
}

impl PhotoEnv {
    pub fn new(app_env: &AppEnv) -> Self {
        Self {
            location_converted: app_env.location_photo_converted.clone(),
            location_original: app_env.location_photo_original.clone(),
            location_watermark: app_env.location_watermark.clone(),
        }
    }

    pub fn get_path(&self, photo: ij::PhotoName) -> String {
        match photo {
            ij::PhotoName::Converted(name) => format!("{}/{}", self.location_converted, name),
            ij::PhotoName::Original(name) => format!("{}/{}", self.location_original, name),
        }
    }
}
// Need to look into this
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PhotoConvertor {
    pub original: String,
    pub converted: String,
}

pub struct Photo {
    pub file_name: String,
    pub data: Bytes,
}

impl PhotoConvertor {
    pub async fn convert_photo(
        original_photo: Photo,
        photo_env: &PhotoEnv,
    ) -> Result<Self, ApiError> {
        // Create file_names
        let original_file_name =
            format!("{}_O_{}.jpg", original_photo.file_name, gen_random_hex(16));
        let converted_file_name =
            format!("{}_C_{}.jpg", original_photo.file_name, gen_random_hex(16));

        let converted_output_location =
            format!("{}/{}", photo_env.location_converted, converted_file_name);

        // Save original to disk
        if tokio::fs::write(
            format!("{}/{}", photo_env.location_original, original_file_name),
            &original_photo.data,
        )
        .await
        .is_err()
        {
            return Err(ApiError::Internal(
                "Unable to save original image".to_owned(),
            ));
        }

        let location_watermark = photo_env.location_watermark.clone();
        tokio::task::spawn_blocking(move || -> Result<Self, ApiError> {
            // Load original into memory, so can manipulate
            let img = image::load_from_memory_with_format(
                &original_photo.data,
                image::ImageFormat::Jpeg,
            )?;

            // Resize image so that one size is max 1000, and other side scales accordingly
            let mut converted_img = img.resize(1000, 1000, image::imageops::FilterType::Nearest);

            // Put the water mark in the bottom right, with a 4px padding
            let watermark = image::open(location_watermark)?;
            let watermark_x = i64::from(converted_img.width() - watermark.width() - 4);
            let watermark_y = i64::from(converted_img.height() - watermark.height() - 4);
            image::imageops::overlay(&mut converted_img, &watermark, watermark_x, watermark_y);

            // save converted to disk at 80% jpg quality
            match File::create(converted_output_location) {
                Ok(output) => {
                    let mut encoder =
                        image::codecs::jpeg::JpegEncoder::new_with_quality(output, 80);

                    encoder.encode(
                        converted_img.as_bytes(),
                        converted_img.width(),
                        converted_img.height(),
                        converted_img.color(),
                    )?;

                    Ok(Self {
                        original: original_file_name,
                        converted: converted_file_name,
                    })
                }
                Err(e) => {
                    error!(%e);
                    Err(ApiError::Internal(
                        "Unable to save converted image".to_owned(),
                    ))
                }
            }
        })
        .await?
    }
}
