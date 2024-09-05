use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use webp::Encoder;
use arboard::Clipboard;

// もし使う場合はここから下を修正

// 定数の定義
const DEFAULT_WIDTH: u32 = 640;
const DEFAULT_HEIGHT: u32 = 480;
const DOWNLOADS_DIR: &str = "Downloads";
const WEBP_QUALITY: f32 = 65.0;

// GitHub用のURLフォーマットを生成する関数
fn generate_github_url(image_name: &str, width: &str, height: &str) -> String {
    format!("![{name}](https://raw.githubusercontent.com/shunkat/image-resize-convert-uploader/master/{width}_{height}/{name}.webp)", name = image_name, width = width, height = height)
}


// ここまで修正

fn main() -> io::Result<()> {
    let newest_image = get_newest_image()?;

    if let Some(image_path) = newest_image {
        println!("最新の画像: {}", image_path.display());
        display_image(&image_path);

        let (new_name, width, height) = get_user_input(&image_path);

        let resized_image = resize_image(&image_path, width, height)?;
        convert_to_webp(&resized_image, width, height)?;
    


        // `resized_image.png` を削除
        fs::remove_file(resized_image)?;

        // クリップボードに GitHub 用のリンク形式をコピー
        let github_link = generate_github_url(&new_name, &width.to_string(), &height.to_string());
        copy_to_clipboard(&github_link)?;
    } else {
        println!("ダウンロードフォルダに画像が見つかりませんでした。");
    }

    Ok(())
}

fn get_newest_image() -> io::Result<Option<PathBuf>> {
    let home = env::var("HOME").expect("HOMEディレクトリの取得に失敗しました");
    let downloads_path = PathBuf::from(home).join(DOWNLOADS_DIR); // 定数を使用

    let entries = fs::read_dir(downloads_path)?;

    let newest_image = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            let is_image = match path.extension() {
                Some(ext) => matches!(ext.to_str()?, "jpg" | "jpeg" | "png" | "gif" | "PNG" | "JPG" | "JPEG"),
                None => false,
            };
            if is_image {
                Some((fs::metadata(&path).ok()?.modified().ok()?, path))
            } else {
                None
            }
        })
        .max_by_key(|&(date, _)| date)
        .map(|(_, path)| path);

    Ok(newest_image)
}

fn get_user_input(image_path: &Path) -> (String, u32, u32) {
    let default_name = image_path.file_stem().unwrap().to_str().unwrap().to_string();

    print!("新しい画像名を入力してください（デフォルト: {}）: ", default_name);
    io::stdout().flush().unwrap();
    let mut new_name = String::new();
    io::stdin().read_line(&mut new_name).unwrap();
    let new_name = new_name.trim().to_string();
    let new_name = if new_name.is_empty() { default_name } else { new_name };

    print!("画像の幅を入力してください（デフォルト: {}）: ", DEFAULT_WIDTH); // 定数を使用
    io::stdout().flush().unwrap();
    let mut width = String::new();
    io::stdin().read_line(&mut width).unwrap();
    let width: u32 = width.trim().parse().unwrap_or(DEFAULT_WIDTH); // 定数を使用

    print!("画像の高さを入力してください（デフォルト: {}）: ", DEFAULT_HEIGHT); // 定数を使用
    io::stdout().flush().unwrap();
    let mut height = String::new();
    io::stdin().read_line(&mut height).unwrap();
    let height: u32 = height.trim().parse().unwrap_or(DEFAULT_HEIGHT); // 定数を使用

    (new_name, width, height)
}

fn resize_image(image_path: &Path, width: u32, height: u32) -> io::Result<PathBuf> {
    let img = image::open(image_path).expect("Failed to open image");
    let resized = img.resize(width, height, image::imageops::FilterType::Triangle);
    let output_path = image_path.with_file_name("resized_image.png");
    resized.save(&output_path).expect("Failed to save resized image");
    Ok(output_path)
}

fn convert_to_webp(image_path: &Path, width: u32, height: u32) -> io::Result<PathBuf> {
    let img = image::open(image_path).expect("Failed to open image");

    // `{width}_{height}` フォルダを作成
    let output_dir = PathBuf::from(format!("{}_{}", width, height));
    fs::create_dir_all(&output_dir)?;

    // フォルダ内に `.webp` 画像を保存
    let output_path = output_dir.join(image_path.with_extension("webp").file_name().unwrap());

    let encoder = Encoder::from_image(&img).expect("Failed to create WebP encoder");
    let webp = encoder.encode(WEBP_QUALITY); // 定数を使用

    // WebPMemory から Vec<u8> に変換
    let webp_data: Vec<u8> = webp.to_vec();

    fs::write(&output_path, webp_data).expect("Failed to write WebP image");
    Ok(output_path)
}

fn rename_image(image_path: &Path, new_name: &str) -> io::Result<PathBuf> {
    let new_path = image_path.with_file_name(format!("{}.webp", new_name));
    fs::rename(image_path, &new_path)?;
    Ok(new_path)
}

fn copy_to_clipboard(text: &str) -> io::Result<()> {
    let mut clipboard = Clipboard::new().expect("Failed to initialize clipboard");
    clipboard.set_text(text).expect("Failed to copy to clipboard");
    Ok(())
}

// TODO: ターミナル内にプレビュー画像表示
fn display_image(_image_path: &Path) {
    // let img = image::open(image_path).expect("Failed to open image");
    // let (width, height) = img.dimensions();
    
    // // 画像を ASCII アートに変換
    // let ascii_art = img_to_ascii(&img, width.min(80) as usize); // 幅を最大80文字に制限
    
    // println!("画像プレビュー (ASCII art):");
    // println!("{}", ascii_art);
    // println!("元の画像サイズ: {}x{}", width, height);
}

// fn img_to_ascii(img: &DynamicImage, width: usize) -> String {
//     let img = img.resize(width as u32, (width as f32 / img.aspect_ratio()) as u32, image::imageops::FilterType::Nearest);
//     let img = img.grayscale();
    
//     let ascii_chars = " .:-=+*#%@";
//     let mut ascii_art = String::new();
    
//     for y in 0..img.height() {
//         for x in 0..img.width() {
//             let pixel = img.get_pixel(x, y);
//             let intensity = pixel[0] as usize * ascii_chars.len() / 256;
//             ascii_art.push(ascii_chars.chars().nth(intensity).unwrap());
//         }
//         ascii_art.push('\n');
//     }
    
//     ascii_art
// }