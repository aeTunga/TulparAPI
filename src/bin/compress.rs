use lz4_flex::frame::FrameEncoder;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct ContentItem {
    id: String,
    title: String,
    body: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ContentCollection {
    id: String,
    name: String,
    items: Vec<ContentItem>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("--demo") => create_demo_data()?,
        Some(input_path) => {
            let output_path = args.get(2).map(|s| s.as_str());
            compress_file(input_path, output_path)?;
        }
        None => {
            eprintln!("Usage:");
            eprintln!("  cargo run --bin compress -- <input.json> [output.json.lz4]");
            eprintln!("  cargo run --bin compress -- --demo");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn compress_file(input_path: &str, output_path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let input = Path::new(input_path);
    if !input.exists() {
        return Err(format!("Input file not found: {}", input_path).into());
    }

    let output = match output_path {
        Some(p) => p.to_string(),
        None => format!("{}.lz4", input_path),
    };

    let json_data = fs::read(input)?;

    let output_file = File::create(&output)?;
    let mut encoder = FrameEncoder::new(output_file);
    encoder.write_all(&json_data)?;
    encoder.finish()?;

    let original_size = json_data.len();
    let compressed_size = fs::metadata(&output)?.len() as usize;
    let ratio = (compressed_size as f64 / original_size as f64) * 100.0;

    println!("Compressed: {} -> {}", input_path, output);
    println!("  Original:   {} bytes", original_size);
    println!("  Compressed: {} bytes", compressed_size);
    println!("  Ratio:      {:.1}%", ratio);

    Ok(())
}

fn create_demo_data() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("data")?;
    fs::create_dir_all("storage/collections")?;

    let demo = ContentCollection {
        id: "rubaiyat".to_string(),
        name: "Rubaiyat of Omar Khayyam".to_string(),
        items: vec![
            ContentItem {
                id: "1".to_string(),
                title: "Quatrain I".to_string(),
                body: "Awake! for Morning in the Bowl of Night...".to_string(),
            },
            ContentItem {
                id: "2".to_string(),
                title: "Quatrain II".to_string(),
                body: "Dreaming when Dawn's Left Hand was in the Sky...".to_string(),
            },
        ],
    };

    let json_path = "data/rubaiyat.json";
    let json_data = serde_json::to_string_pretty(&demo)?;
    if !Path::new(json_path).exists() {
        fs::write(json_path, &json_data)?;
        println!("Created: {}", json_path);
    } else {
         println!("Skipped creation of {} (already exists)", json_path);
    }

    let lz4_path = "storage/collections/rubaiyat.json.lz4";
    let output_file = File::create(lz4_path)?;
    let mut encoder = FrameEncoder::new(output_file);
    let full_json_data = fs::read(json_path)?;
    encoder.write_all(&full_json_data)?;
    encoder.finish()?;
    println!("Created: {}", lz4_path);

    println!("\nDemo data ready. Start server and test:");
    println!("  curl http://localhost:3000/api/v1/collections/rubaiyat");

    Ok(())
}
