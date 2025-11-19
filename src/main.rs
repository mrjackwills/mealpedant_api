#![allow(unused, dead_code)]

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod api_error;
mod argon;
mod database;
mod emailer;
mod helpers;
mod macros;
mod parse_env;
mod photo_convertor;
mod scheduler;
mod servers;

use std::io::{Read, Write};

use api_error::ApiError;

use libwebp::WebPEncodeRGB;
use parse_env::AppEnv;
use scheduler::BackupSchedule;
use servers::{api, static_serve};
use tracing_subscriber::{fmt, prelude::__tracing_subscriber_SubscriberExt};

fn setup_tracing(app_envs: &AppEnv) -> Result<(), ApiError> {
    let logfile = tracing_appender::rolling::never(&app_envs.location_logs, "api.log");

    let log_fmt = fmt::Layer::default().json().with_writer(logfile);

    match tracing::subscriber::set_global_default(
        fmt::Subscriber::builder()
            .with_file(true)
            .with_line_number(true)
            .with_max_level(app_envs.log_level)
            .finish()
            .with(log_fmt),
    ) {
        Ok(()) => Ok(()),
        Err(e) => {
            println!("{e:?}");
            Err(ApiError::Internal(S!("Unable to start tracing")))
        }
    }
}

async fn start(app_env: AppEnv) -> Result<(), ApiError> {
    tracing::info!(
        "{} - {} - {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        app_env.run_mode
    );
    let postgres = database::db_postgres::db_pool(&app_env).await?;
    let redis = database::DbRedis::get_pool(&app_env).await?;
    BackupSchedule::init(&app_env);

    let static_data = (C!(app_env), C!(postgres), C!(redis));
    tokio::spawn(async move {
        if let Err(e) =
            static_serve::StaticRouter::serve(static_data.0, static_data.1, static_data.2).await
        {
            tracing::error!("{e}");
        }
    });
    tokio::spawn(api::serve(app_env, postgres, redis))
        .await
        .ok();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let app_env = parse_env::AppEnv::get_env();

    if let Err(e) = setup_tracing(&app_env) {
        println!("tracing error: {e}");
        std::process::exit(1);
    }
    tokio::spawn(start(app_env)).await.ok();
    Ok(())
}

// go through every file in local_location/static/converted and convert from jpg to webp,
// ideally should start with the original, so instead use a list of images and open original then convert, but need to keep original filename

// select * from meal_photo, then for each open original, covnert to webp, save in folder with 1234/56/123456[...].webp name

// convert to webp, delete converted,
// update original

// #[derive(Debug, sqlx::FromRow)]
// struct MealPhoto {
//     meal_photo_id: i64,
//     photo_original: String,
//     photo_converted: String,
// }

// fn create_dirs(name: &str) -> Result<String, ApiError> {
//     let t = name.chars().take(4).collect::<String>();
//     // let v = name.chars().skip(4).take(3).collect::<String>();

//     let dirs = format!("/workspaces/backend/location_local/static/converted/webp/{t}");

//     std::fs::create_dir_all(&dirs).unwrap();

//     Ok(format!(
//         "{dirs}/{}.webp",
//         name.split_once('.').unwrap_or_default().0
//     ))
// }

// fn convert_image(og_photo: &[u8], new_path: &str) -> Result<(), ApiError> {
//     let img = image::load_from_memory_with_format(og_photo, image::ImageFormat::Jpeg)?;

//     let mut converted_img = img.resize(1000, 1000, image::imageops::FilterType::Nearest);
//     let watermark = image::open("/workspaces/backend/docker/data/watermark.png")?;
//     let watermark_x = i64::from(converted_img.width() - watermark.width() - 4);
//     let watermark_y = i64::from(converted_img.height() - watermark.height() - 4);
//     image::imageops::overlay(&mut converted_img, &watermark, watermark_x, watermark_y);

//     let buf = WebPEncodeRGB(
//         converted_img.as_bytes(),
//         converted_img.width(),
//         converted_img.height(),
//         converted_img.width() * 3,
//         75.0,
//     )?
//     .to_vec();
//     let mut f = std::fs::File::create(new_path).unwrap();
//     f.write_all(&buf).unwrap();

//     // wrtie to disk with new same name
//     Ok(())
// }
// async fn fix_photos() -> Result<(), ApiError> {
//     let app_env = parse_env::AppEnv::get_env();
//     let postgres = database::db_postgres::db_pool(&app_env).await?;

//     let photos = sqlx::query_as!(
//         MealPhoto,
//         "SELECT meal_photo_id, photo_original, photo_converted FROM meal_photo"
//     )
//     .fetch_all(&postgres)
//     .await?;

//     for (index, i) in photos.iter().enumerate() {
//         let mut og = std::fs::File::open(format!(
//             "/workspaces/backend/location_local/static/original/{}",
//             i.photo_original
//         ))
//         .unwrap();
//         let mut gg = vec![];
//         og.read_to_end(&mut gg).unwrap();

//         let new_path = create_dirs(&i.photo_converted).unwrap();
//         convert_image(&gg, &new_path).unwrap();
//         println!("done {:<03}/{:<03}", index + 1, photos.len());

//         // println!("{gg:?}");
//         // println!("{i:?}");
//     }

//     // println!("{photos:#?}");

//     Ok(())
// }

// fn organize_by_prefix() -> Result<(), ()> {
//     let root = std::path::Path::new("/workspaces/backend/location_local/static/original");
//     // 1. Read every entry in the directory
//     for entry in std::fs::read_dir(&root).unwrap() {
//         let entry = entry.unwrap();
//         let path = entry.path();

//         // skip directories, process files only
//         if path.is_dir() {
//             continue;
//         }

//         // 2. Grab the first 4 chars of the file-stem
//         let stem = path
//             .file_stem()
//             .and_then(|s| s.to_str()) // OsStr → &str
//             .and_then(|s| s.get(0..4)) // first 4 chars
//             .map(str::to_owned);

//         if let Some(prefix) = stem {
//             // 3. Create target folder (idempotent)
//             let target_dir = root.join(&prefix);
//             // println!("create: {target_dir:?}");
//             std::fs::create_dir_all(&target_dir).unwrap();

//             // 4. Move the file, keeping its original name
//             let new_path = target_dir.join(path.file_name().unwrap());
//             // println!("rename: {path:?} to {new_path:?}");
//             std::fs::rename(&path, new_path).unwrap();
//         }
//     }
//     Ok(())
// }
