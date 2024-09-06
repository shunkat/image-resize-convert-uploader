use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use webp::Encoder;
use arboard::Clipboard;

const DEFAULT_WIDTH: u32 = 640;
const DEFAULT_HEIGHT: u32 = 480;
const DOWNLOADS_DIR: &str = "Downloads";
const WEBP_QUALITY: f32 = 65.0;

fn generate_github_url(image_name: &str, width: &str, height: &str) -> String {
    format!("![{name}](https://raw.githubusercontent.com/shunkat/image-resize-convert-uploader/master/{width}_{height}/{name}.webp)", name = image_name, width = width, height = height)
}

fn main() -> io::Result<()> {
    println!("ダウンロードフォルダから最新の画像を検索中...");
    let newest_image = get_newest_image()?;

    if let Some(image_path) = newest_image {
        println!("最新の画像を発見: {}", image_path.display());
        display_image(&image_path);

        println!("ユーザーからの入力を待機中...");
        let (new_name, width, height) = get_user_input(&image_path);

        println!("画像をリサイズ中...");
        let resized_image = resize_image(&image_path, width, height, &new_name)?;
        println!("画像のリサイズが完了しました: {}", resized_image.display());

        println!("WebP形式に変換中...");
        let webp_image = convert_to_webp(&resized_image, width, height)?;
        println!("WebP形式に変換が完了しました: {}", webp_image.display());

        // `resized_image.png` を削除
        println!("リサイズ済みPNG画像を削除中...");
        fs::remove_file(resized_image)?;
        println!("削除が完了しました。");

        let github_link = generate_github_url(&new_name, &width.to_string(), &height.to_string());
        println!("GitHubリンクを生成しました: {}", github_link);

        println!("クリップボードにコピー中...");
        copy_to_clipboard(&github_link)?;
        println!("クリップボードへのコピーが完了しました。");

    } else {
        println!("ダウンロードフォルダに画像が見つかりませんでした。");
    }

    Ok(())
}

fn get_newest_image() -> io::Result<Option<PathBuf>> {
    let home = env::var("HOME").expect("HOMEディレクトリの取得に失敗しました");
    let downloads_path = PathBuf::from(home).join(DOWNLOADS_DIR);

    println!("ダウンロードフォルダ: {}", downloads_path.display());
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

    print!("画像の幅を入力してください（デフォルト: {}）: ", DEFAULT_WIDTH);
    io::stdout().flush().unwrap();
    let mut width = String::new();
    io::stdin().read_line(&mut width).unwrap();
    let width: u32 = width.trim().parse().unwrap_or(DEFAULT_WIDTH);

    print!("画像の高さを入力してください（デフォルト: {}）: ", DEFAULT_HEIGHT);
    io::stdout().flush().unwrap();
    let mut height = String::new();
    io::stdin().read_line(&mut height).unwrap();
    let height: u32 = height.trim().parse().unwrap_or(DEFAULT_HEIGHT);

    println!("入力が完了しました: 名前 = {}, 幅 = {}, 高さ = {}", new_name, width, height);

    (new_name, width, height)
}

fn resize_image(image_path: &Path, width: u32, height: u32, new_name: &str) -> io::Result<PathBuf> {
    let img = image::open(image_path).expect("画像のオープンに失敗しました");

    let resized = img.resize(width, height, image::imageops::FilterType::Triangle);

    // 新しいファイル名に new_name を使用して保存する
    let output_file_name = format!("{}_resized.png", new_name);

    // 画像が保存されるパスを作成する
    let output_path = image_path.with_file_name(output_file_name);

    resized.save(&output_path).expect("リサイズ画像の保存に失敗しました");

    Ok(output_path)
}

fn convert_to_webp(image_path: &Path, width: u32, height: u32) -> io::Result<PathBuf> {
    let img = image::open(image_path).expect("Failed to open image");

    // `{width}_{height}` フォルダを作成
    let output_dir = PathBuf::from(format!("{}_{}", width, height));
    fs::create_dir_all(&output_dir)?;

    let output_path = output_dir.join(image_path.with_extension("webp").file_name().unwrap());

    let encoder = Encoder::from_image(&img).expect("WebPエンコーダーの生成に失敗しました");
    let webp = encoder.encode(WEBP_QUALITY);

    let webp_data: Vec<u8> = webp.to_vec();

    fs::write(&output_path, webp_data).expect("WebP画像の書き込みに失敗しました");
    Ok(output_path)
}

fn copy_to_clipboard(text: &str) -> io::Result<()> {
    let mut clipboard = Clipboard::new().expect("クリップボードの初期化に失敗しました");
    clipboard.set_text(text).expect("クリップボードへのコピーに失敗しました");
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