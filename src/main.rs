extern crate json;
extern crate image;
extern crate ftp;

use ftp::FtpStream;
use image::io::Reader as ImageReader;
use image::GenericImageView;
use image::imageops::FilterType::Triangle;

use std::{fs,io};
use std::io::{Cursor, Write};
use std::process::exit;

fn check_json_null(name: &str, value: &json::JsonValue) {
    if *value == json::Null {
        eprintln!("[ERROR]: Could not parse field \"{}\" in file `settings.json`.\nAborting.", name);
        exit(1);
    }
}

fn clean_string(s: String) -> String {
    s.replace(" ", "")
    .replace("\t", "")
    .replace("\n", "")
    .replace("\r", "")
}

fn get_string(settings: &json::JsonValue, key: &str) -> String {
    let jv = &settings[key];
    check_json_null(key, jv);
    if let json::JsonValue::Short(v) = jv {
        return clean_string(v.to_owned().to_string());
        
    }
    eprintln!("[ERROR]: Field \"{}\" in file `settings.json` is supposed to be a string.\nAborting.", key);
    exit(1);
}

fn get_data(settings: &json::JsonValue) -> (String, u64, u64, u64) {
    let jv = &settings["data"];
    check_json_null("data", jv);
    if let json::JsonValue::Object(data) = jv {
        if let json::JsonValue::Number(giorno) = data["giorno"] {
            if let json::JsonValue::Number(mese) = data["mese"] {
                if let json::JsonValue::Number(anno) = data["anno"] {
                    let s = format!("{}{:02}{:02}", anno.as_parts().1 % 100, mese.as_parts().1 % 100, giorno.as_parts().1 % 100); 
                    if s.len() == 6 {
                        return (s, anno.as_parts().1, mese.as_parts().1, giorno.as_parts().1);
                    }
                }
            }
        }
        
    }

    eprintln!("[ERROR]: Field \"data\" in file `settings.json` is invalid.\nAborting.");
    exit(1);
}

fn find_images() -> Vec<std::path::PathBuf> {
    let mut images: Vec<std::path::PathBuf> = Vec::new();
    for element in std::path::Path::new(".").read_dir().unwrap() {
        let path = element.unwrap().path();
        if let Some(extension) = path.extension() {
            if extension == "jpeg" || extension == "jpg" || extension == "JPG" || extension == "png" {
                images.push(path);
            }
        }
    }
    
    images.sort();

    return images;
}

fn find_files(dir: &str) -> Vec<std::path::PathBuf> {
    let mut paths: Vec<std::path::PathBuf> = Vec::new();
    for element in std::path::Path::new(dir).read_dir().unwrap() {
        let path = element.unwrap().path();
        paths.push(path);
    }
    
    paths.sort();

    return paths;
}

fn upload_dir(stream: &mut FtpStream, dir: &str) {
    println!("+ Uploading DIR `{}`...", dir);
    stream.mkdir(&dir).unwrap();
    println!("+ [FTP]: mkdir {}", &dir);
    // stream.cwd(dir).unwrap();
    stream.transfer_type(ftp::types::FileType::Image).unwrap();
    for file in find_files(dir) {
        print!("  - Uploading `{}`...", file.to_str().unwrap());
        io::stdout().flush().unwrap();

        let content = fs::read(file.clone()).unwrap();
        let mut reader = Cursor::new(content);

        stream.put(&file.to_str().unwrap(), &mut reader).unwrap();
        println!(" done!");
    }
}

fn main() {
    println!("--- SETTINGS ---");


    print!("+ Searching for configuration file: `settings.json`...");
    io::stdout().flush().unwrap();

    let settings_file = fs::read_to_string("settings.json")
        .expect("[ERROR]: Could not find `settings.json`.\nAborting.");
    
    println!(" done!");
    print!("+ Parsing `settings.json`...");
    io::stdout().flush().unwrap();

    let settings = json::parse(&settings_file)
        .expect("[ERROR]: Could not parse `settings.json`.\nAborting.");

    let titolo = get_string(&settings, "titolo");
    let branca = get_string(&settings, "branca").to_uppercase();
    let server = get_string(&settings, "server");
    let utente = get_string(&settings, "utente");
    let password = get_string(&settings, "password");

    let (data, anno, mese, _) = get_data(&settings);

    println!(" done!");

    println!();
    println!("--- IMAGES ---");

    
    let dir_path = format!("{}_{}_{}", data, branca, titolo);
    print!("+ Creating dir `{}`...", dir_path);
    io::stdout().flush().unwrap();
    let _ = fs::remove_dir_all(dir_path.clone());
    fs::create_dir(dir_path.clone()).unwrap();
    println!(" done!");


    let images = find_images();

    for (n, path) in images.iter().enumerate() {
        print!("  + Processing `{:?}`... ", path);
        std::io::stdout().flush().unwrap();

        let img = ImageReader::open(path).unwrap().decode().unwrap();
        let size = img.dimensions();

        let img_scaled;
        if size.0 > size.1 {
            img_scaled = img.resize_to_fill(800, 600, Triangle);
        } else {
            img_scaled = img.resize_to_fill(600, 800, Triangle);
        }

        let new_name = format!("{}/{}_{}_{}_{:03}.JPG", dir_path, data, branca, titolo, n + 1);

        img_scaled.save(new_name.clone()).expect("[ERROR]: Could not save image.\nAborting.");
        println!(" done!\n    -> Saved as `{}`!", new_name);
    }

    println!();
    print!("Upload photos? [Y]/n ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Could not understand input");

    let answer = input.trim();

    if !(answer == "Y" || answer == "y" || answer == "yes" || answer == "YES" || answer == "Yes") {
        println!("Ok. The photos are not going to be uploaded.\nExiting...");
        exit(0);
    }

    println!();
    println!("--- FTP ---");

    let mut ftp_stream = FtpStream::connect(format!("{}:21", server)).unwrap();
    ftp_stream.login(&utente, &password).unwrap();

    if mese < 8 {
        let dir = format!("{}-{}", anno - 1, anno);
        ftp_stream.cwd(&dir).unwrap();
        println!("[FTP]: cd {}/", dir);
    } else {
        let dir = format!("{}-{}", anno, anno + 1);
        ftp_stream.cwd(&dir).unwrap();
        println!("[FTP]: cd {}/", dir);
    }

    upload_dir(&mut ftp_stream, &dir_path);

    let _ = ftp_stream.quit();

}
