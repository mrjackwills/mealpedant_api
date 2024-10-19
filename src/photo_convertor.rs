use bytes::Bytes;
use std::fs::File;
use tracing::error;

use crate::api::ij;
use crate::api_error::ApiError;
use crate::helpers::gen_random_hex;
use crate::parse_env::AppEnv;
use crate::{C, S};

#[derive(Debug, Clone)]
pub struct PhotoLocationEnv {
    original: String,
    converted: String,
    watermark: String,
}

impl PhotoLocationEnv {
    pub fn new(app_env: &AppEnv) -> Self {
        Self {
            converted: C!(app_env.location_photo_converted),
            original: C!(app_env.location_photo_original),
            watermark: C!(app_env.location_watermark),
        }
    }

    pub fn get_path(&self, photo: ij::PhotoName) -> String {
        match photo {
            ij::PhotoName::Converted(name) => format!("{}/{name}", self.converted),
            ij::PhotoName::Original(name) => format!("{}/{name}", self.original),
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
        photo_env: &PhotoLocationEnv,
    ) -> Result<Self, ApiError> {
        // Create file_names
        let original_file_name =
            format!("{}_O_{}.jpg", original_photo.file_name, gen_random_hex(16));
        let converted_file_name =
            format!("{}_C_{}.jpg", original_photo.file_name, gen_random_hex(16));

        let converted_output_location = format!("{}/{converted_file_name}", photo_env.converted);

        // Save original to disk
        if tokio::fs::write(
            format!("{}/{original_file_name}", photo_env.original),
            &original_photo.data,
        )
        .await
        .is_err()
        {
            return Err(ApiError::Internal(S!("Unable to save original image")));
        }

        let location_watermark = C!(photo_env.watermark);
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
                        converted_img.color().into(),
                    )?;

                    Ok(Self {
                        original: original_file_name,
                        converted: converted_file_name,
                    })
                }
                Err(e) => {
                    error!(%e);
                    Err(ApiError::Internal(S!("Unable to save converted image")))
                }
            }
        })
        .await?
    }
}
