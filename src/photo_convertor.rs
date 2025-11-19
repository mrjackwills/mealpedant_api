use bytes::Bytes;
use image::EncodableLayout;
use libwebp::WebPEncodeRGB;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

use crate::api_error::ApiError;
use crate::parse_env::AppEnv;
use crate::servers::ij;
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

    pub fn get_original_path(&self) -> PathBuf {
        PathBuf::from(&self.original)
    }

    pub fn get_converted_path(&self) -> PathBuf {
        PathBuf::from(&self.converted)
    }

    pub fn get_pathbuff(&self, photo: ij::PhotoName) -> PathBuf {
        match photo {
            ij::PhotoName::Converted(name) => PathBuf::from(&self.converted)
                .join(name.chars().take(4).collect::<String>())
                .join(name),
            ij::PhotoName::Original(name) => PathBuf::from(&self.original)
                .join(name.chars().take(4).collect::<String>())
                .join(name),
        }
    }
}
// Need to look into this
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PhotoConvertor {
    pub original: String,
    pub converted: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Photo {
    pub file_name: String,
    pub data: Bytes,
}

impl PhotoConvertor {
    /// Write bytes to disk, create dir if doesn't exist
    async fn write_to_disk(filepath: PathBuf, data: &[u8]) -> Result<(), ApiError> {
        if let Some(dir) = filepath.parent() {
            tokio::fs::create_dir_all(dir).await?;
        }
        let mut file = tokio::fs::File::create_new(filepath).await?;
        file.write_all(data).await?;
        file.flush().await?;
        Ok(())
    }

    /// Generate a random file name for a photo,
    /// 32/33 chars include .jpg/.webp, first 26 is a ulid, then 1/0 depending on person, then is 1/0 depending if original, finally .jpg for original and .webp for converted
    /// [ulid:26][Dave/Jack][Original,Converted].[jpg/webp]
    /// [ulid:26][0/1]      [0/1]               .[jpg/webp]
    fn generate_name(name: &str, original: bool) -> String {
        format!(
            "{ulid}{person}{variant}.{file_type}",
            ulid = ulid::Ulid::new().to_string().to_lowercase(),
            person = i8::from(name != "D"),
            variant = i8::from(!original),
            file_type = if original { "jpg" } else { "webp" }
        )
    }

    /// Get the 4 char dir name from a filename
    fn dir_name(file_name: &str) -> String {
        file_name.chars().take(4).collect()
    }

    pub async fn convert_photo(
        original_photo: Photo,
        photo_env: &PhotoLocationEnv,
    ) -> Result<Self, ApiError> {
        let original_file_name = Self::generate_name(&original_photo.file_name, true);
        let converted_file_name = Self::generate_name(&original_photo.file_name, false);

        let original_bytes = original_photo.data.as_bytes().to_vec();

        let location_watermark = C!(photo_env.watermark);
        let converted_bytes = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, ApiError> {
            let img =
                image::load_from_memory_with_format(&original_bytes, image::ImageFormat::Jpeg)?;

            let mut converted_img = img.resize(1000, 1000, image::imageops::FilterType::Nearest);
            let watermark = image::open(location_watermark)?;
            let watermark_x = i64::from(converted_img.width() - watermark.width() - 4);
            let watermark_y = i64::from(converted_img.height() - watermark.height() - 4);
            image::imageops::overlay(&mut converted_img, &watermark, watermark_x, watermark_y);

            Ok(WebPEncodeRGB(
                converted_img.as_bytes(),
                converted_img.width(),
                converted_img.height(),
                converted_img.width() * 3,
                75.0,
            )?
            .to_vec())
        })
        .await??;

        tokio::try_join!(
            Self::write_to_disk(
                PathBuf::from(&photo_env.original)
                    .join(Self::dir_name(&original_file_name))
                    .join(&original_file_name),
                original_photo.data.as_bytes()
            ),
            Self::write_to_disk(
                PathBuf::from(&photo_env.converted)
                    .join(Self::dir_name(&converted_file_name))
                    .join(&converted_file_name),
                &converted_bytes,
            )
        )
        .map_err(|_| ApiError::Internal(S!("Unable to save original image")))?;

        Ok(Self {
            original: original_file_name,
            converted: converted_file_name,
        })
    }
}
