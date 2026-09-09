use image::{ImageBuffer, ImageFormat, Rgb};
use qrcode::{Color, EcLevel, QrCode};
use std::{env, error::Error, path::PathBuf};

const WIDTH: u32 = 170;
const HEIGHT: u32 = 170;
const QUIET_ZONE: u32 = 4;

fn main() -> Result<(), Box<dyn Error>> {
    // Re-run build.rs when .env changes.
    println!("cargo:rerun-if-changed=.env");

    // Load .env.
    dotenvy::dotenv().ok();

    // Get the value to encode.
    let qr_data = env::var("BOARD_NAME").map_err(|_| "BOARD_NAME is not set in .env")?;
    println!("cargo:rustc-env=BOARD_NAME={qr_data}");

    // Generate QR code.
    let qr = QrCode::with_error_correction_level(qr_data.as_bytes(), EcLevel::M)?;

    let modules = qr.width() as u32;

    // Calculate the largest integer module size that fits.
    let available = WIDTH - 2 * QUIET_ZONE;
    let module_size = available / modules;

    if module_size == 0 {
        return Err(format!(
            "QR code has {} modules and cannot fit into {}x{}",
            modules, WIDTH, HEIGHT
        )
        .into());
    }

    let qr_size = modules * module_size;

    // Center the QR code.
    let offset_x = (WIDTH - qr_size) / 2;
    let offset_y = (HEIGHT - qr_size) / 2;

    // Create an RGB image.
    let mut image = ImageBuffer::from_pixel(WIDTH, HEIGHT, Rgb([255u8, 255u8, 255u8]));

    // Draw QR modules.
    for y in 0..modules {
        for x in 0..modules {
            let color = match qr[(x as usize, y as usize)] {
                Color::Dark => Rgb([0u8, 0u8, 0u8]),
                Color::Light => Rgb([255u8, 255u8, 255u8]),
            };

            let px = offset_x + x * module_size;
            let py = offset_y + y * module_size;

            for dy in 0..module_size {
                for dx in 0..module_size {
                    image.put_pixel(px + dx, py + dy, color);
                }
            }
        }
    }

    // Put generated file in Cargo's build output directory.
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let output = out_dir.join("qr.bmp");

    image.save_with_format(&output, ImageFormat::Bmp)?;

    println!("cargo:warning=Generated QR code: {}", output.display());

    let ssid = env::var("WIFI_SSID").expect("WIFI_SSID not set");
    println!("cargo:rustc-env=WIFI_SSID={ssid}");
    let password = env::var("WIFI_PASSWORD").expect("WIFI_PASSWORD not set");
    println!("cargo:rustc-env=WIFI_PASSWORD={password}");
    let mqtt_url = env::var("MQTT_URL").expect("MQTT_URL not set");
    println!("cargo:rustc-env=MQTT_URL={mqtt_url}");
    let mqtt_user = env::var("MQTT_USER").expect("MQTT_USER not set");
    println!("cargo:rustc-env=MQTT_USER={mqtt_user}");
    let mqtt_password = env::var("MQTT_PASSWORD").expect("MQTT_PASSWORD not set");
    println!("cargo:rustc-env=MQTT_PASSWORD={mqtt_password}");

    embuild::espidf::sysenv::output();

    Ok(())
}
