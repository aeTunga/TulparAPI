use tulpar_api::{config::Config, db};
use lz4_flex::frame::FrameEncoder;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!("Usage: cargo run --bin seed -- <input_json> <alias> <name> [language]");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let alias = &args[2];
    let name = &args[3];
    let language = args.get(4).cloned();

    println!("Seeding collection:");
    println!("  Input: {}", input_path);
    println!("  Alias: {}", alias);
    println!("  Name:  {}", name);
    if let Some(lang) = &language {
        println!("  Lang:  {}", lang);
    }

    let config = Config::from_env();
    let pool = db::establish_connection(&config.database_url).await?;

    let input = Path::new(input_path);
    if !input.exists() {
        return Err(format!("Input file not found: {}", input_path).into());
    }

    let storage_dir = config.storage_path.join("collections");
    fs::create_dir_all(&storage_dir)?;

    let filename = format!("{}.json.lz4", alias);
    let output_path = storage_dir.join(&filename);

    let json_data = fs::read(input)?;
    let output_file = File::create(&output_path)?;
    let mut encoder = FrameEncoder::new(output_file);
    encoder.write_all(&json_data)?;
    encoder.finish()?;

    println!("Compressed to: {:?}", output_path);

    let file_path_db = format!("storage/collections/{}", filename);
    
    sqlx::query("INSERT INTO collections (alias, name, file_path, language) VALUES (?, ?, ?, ?) ON CONFLICT(alias) DO UPDATE SET name=excluded.name, file_path=excluded.file_path, language=excluded.language")
        .bind(alias)
        .bind(name)
        .bind(file_path_db)
        .bind(language)
        .execute(&pool)
        .await?;

    println!("Database updated successfully.");

    Ok(())
}
