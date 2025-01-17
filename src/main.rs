extern crate chrono;
extern crate json;
extern crate image;
extern crate ftp;
extern crate strum;
extern crate path_slash;
extern crate walkdir;

use chrono::Local;
use ffi::{GetCurrentMonitor, GetMonitorHeight, GetMonitorWidth};
use ftp::FtpStream;
use gui::{check_ctrl_shortcut, draw_outlined_text, gui_check_box, gui_check_box_update, gui_number_input_update, gui_seecret_text_input, gui_text_input, gui_text_input_update, is_key_pressed_repeat};
use image::ImageReader;
use image::{GenericImageView, DynamicImage};
use path_slash::PathBufExt as _;
use image::imageops::FilterType::Lanczos3;
use raylib::ffi::CheckCollisionPointRec;
use raylib::prelude::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use walkdir::WalkDir;

use std::collections::VecDeque;
use std::ffi::{CString, c_void};
use std::path::PathBuf;
use std::time::Duration;
use std::{fmt, fs, io, thread, vec};
use std::io::{Cursor, Write};

mod gui;

const SMALLER_DIMENSION: u32 = 600;
const BIGGER_DIMENSION: u32 = 800;

const HD_SMALLER_DIMENSION: u32 = 1200;
const HD_BIGGER_DIMENSION: u32 = 1600;

const THEME_COLOR: Color = Color::new(85, 138, 255, 255);
const BACKGROUND_COLOR: Color = Color::new(0x18, 0x18, 0x18, 0xff);

fn clean_string(s: String) -> String {
    s.replace(" ", "")
    .replace("\t", "")
    .replace("\n", "")
    .replace("\r", "")
}

fn check_single_image_path(p: PathBuf, images: &mut Vec<PathBuf>){
    match p.try_exists() {
        Ok(true) => {
            if let Some(extension) = p.clone().extension() {
                if extension == "jpeg" || extension == "jpg" || extension == "JPG" || extension == "png" || extension == "PNG" {
                    images.push(p);
                } else if extension == "txt" {
                    let content = fs::read_to_string(p).unwrap_or_default();
                    let nps = content.lines()
                        .collect::<Vec<_>>();

                    for np in check_images_paths(&nps) {
                        images.push(np);
                    }
                }
            }
        },
        _ => {},
    }
}

fn check_images_paths(files: &Vec<&str>) -> Vec<PathBuf> {
    let mut images = vec![];
    for f in files {
        let p = PathBuf::from(f);
        if p.is_dir() {
            let mut entries = WalkDir::new(p)
                                            .into_iter()
                                            .filter_map(|e| e.ok())
                                            .map(|e| PathBuf::from(e.path()))
                                            .collect::<Vec<_>>();
            entries.sort();
            for entry in entries {
                let np = PathBuf::from(entry);
                check_single_image_path(np, &mut images);
            }
        } else {
            check_single_image_path(p, &mut images);
        }
        
    }

    return images;
}


fn save_used_files(path: &str,  images: &Vec<ImgData>) -> io::Result<()> {
    let mut f = fs::File::create(path)?;
    for img in images.iter() {
        writeln!(&mut f, "{}", img.path.display())?;
    }

    Ok(())
}

fn find_files(dir: &str) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for element in std::path::Path::new(dir).read_dir().unwrap() {
        let path = element.unwrap().path();
        paths.push(path);
    }

    paths.sort();

    return paths;
}

struct ImgData {
    path: PathBuf,
    filename: String,
    image: DynamicImage,
    texture: Texture2D,
}

impl ImgData {
    pub fn new(path: PathBuf, filename: String, image: DynamicImage, texture: Texture2D) -> ImgData {
        ImgData {
            path,
            filename,
            image,
            texture
        }
    }
}

#[derive(Debug, EnumIter, Eq, PartialEq, Copy, Clone)]
enum AppTab {
    InputData,
    SelectionLab,
}

impl fmt::Display for AppTab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AppTab::*;
        write!(f, "{}", match self {
            InputData => "Dati",
            SelectionLab => "Foto",
        })
    }
}

enum UploadStatus {
    None,
    CreatingDir,
    SavingImage(usize),
    DoneSaving,
    Connecting,
    UploadingImage(usize),
    Done,
    Error(String),
}

fn draw_tab_buttons(d: &mut RaylibDrawHandle, active_tab: AppTab, w: f32, h: f32, font_size: i32) -> Option<AppTab> {
    let mut next_tab = None;
    let button_count = AppTab::iter().count() as f32;
    let button_width = (w/(button_count+2.0)).max(100.0);
    let button_height = (h/20.0).max(10.0);
    let button_padding = (h/75.0).max(10.0);
    let start_x = w/2.0 - ((button_count * button_width + (button_count-1.0) * button_padding) as f32  * 0.5);

    for (i, e) in AppTab::iter().enumerate() {
        let i = i as f32;
        let rect = rrect(start_x + (button_width+button_padding)*i, font_size as f32 + button_padding * 7.5, button_width, button_height);
        let label = format!("{}", e);
        let label_width = d.measure_text(label.as_str(), font_size);
        
        if d.gui_check_box(rect, None, &mut (e == active_tab)) {
            next_tab = Some(e);
        }       
        d.draw_text(label.as_str(),
            rect.x as i32 + (rect.width as i32 - label_width)/2,
            rect.y as i32 + (rect.height as i32 - font_size)/2,
            font_size,
            if e == active_tab { Color::WHITE } else { Color::GRAY }
        );
    }

    return next_tab;
}

fn get_next_tab(current_tab: AppTab) -> AppTab {
    let mut last_tab = None;
    for at in AppTab::iter().chain(AppTab::iter()) {
        if let Some(last_tab) = last_tab {
            if last_tab == current_tab {
                return at;
            }
        }
        last_tab = Some(at);
    }
    unreachable!();
}

fn gui_app() {
    let (mut rl, thread) = raylib::init()
        .size(720, 540)
        .title("Foto TPM")
        .resizable()
        .vsync()
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    rl.set_exit_key(None);
    rl.set_target_fps(30);

    let monitor_id = unsafe { GetCurrentMonitor() };
    let m_width = unsafe { GetMonitorWidth(monitor_id) };
    let m_height = unsafe { GetMonitorHeight(monitor_id) };
    rl.set_window_size(m_width*2/3, m_height*2/3);
    rl.set_window_position(m_width/6, m_height/6);

    rl.set_window_min_size(640, 480);

    let version_text = format!("v{}", env!("CARGO_PKG_VERSION"));

    let mut app_tab = AppTab::InputData;
    let mut next_tab = app_tab;
    let mut upload = false;
    let mut upload_status = UploadStatus::None;

    let mut w = 0; let mut h = 0;
    let mut font_size = 0;

    let mut file_queue = VecDeque::new();
    let mut last_image_loaded = false;
    let mut images: Vec<ImgData> = Vec::new();

    let mut file_list_scroll_index = 0;
    let mut file_list_active: i32 = 0;
    let mut list_moved_by_key = false;

    let mut titolo_buf = Vec::new();
    let mut branca_buf = Vec::new();
    let mut giorno_buf = Vec::new();
    let mut mese_buf = Vec::new();
    let mut anno_buf = Vec::new();
    let mut server_buf = Vec::new();
    let mut utente_buf = Vec::new();
    let mut pw_buf = Vec::new();

    let mut text_box_width;
    let mut text_box_height;

    let mut titolo_rect = rrect(0.0, 0.0, 0.0, 0. );
    let mut branca_rect = rrect(0.0, 0.0, 0.0, 0. );
    let mut giorno_rect = rrect(0.0, 0.0, 0.0, 0. );
    let mut mese_rect = rrect(0.0, 0.0, 0.0, 0. );
    let mut anno_rect = rrect(0.0, 0.0, 0.0, 0. );
    let mut server_rect = rrect(0.0, 0.0, 0.0, 0. );
    let mut utente_rect = rrect(0.0, 0.0, 0.0, 0. );
    let mut pw_rect = rrect(0.0, 0.0, 0.0, 0. );
    let mut hd_rect = rrect(0.0, 0.0, 0.0, 0. );

    let mut text_box_active = -1;

    let mut titolo = String::default();
    let mut branca = String::default();
    let mut data = String::default();
    let mut mese = 0;
    let mut anno = 2000;
    let mut server = String::default();
    let mut utente = String::default();
    let mut password = String::default();

    let mut hd_images = false;

    let mut image_dir = String::default();

    let mut ftp_stream = None;
    let mut files_to_upload = Vec::new();

    while !rl.window_should_close() {
        let new_files = check_images_paths(&rl.load_dropped_files().paths());
        file_queue.append(&mut new_files.into());

        if !upload {
            app_tab = next_tab;

            // Update
            match app_tab {
                AppTab::InputData => {
                    if check_ctrl_shortcut(&rl, Some(KeyboardKey::KEY_TAB)) {
                        next_tab = get_next_tab(app_tab);
                    } else if rl.is_key_pressed(KeyboardKey::KEY_TAB) || is_key_pressed_repeat(KeyboardKey::KEY_TAB)  {
                        let delta = if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) || rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT) {
                            -1
                        } else {
                            1
                        };
                        text_box_active = (9 + text_box_active + delta) % 9;
                    }
                    
                    if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                        text_box_active = -1;
                    }

                    text_box_width = 500f32.max(w as f32 / 3.0);
                    let mut idx = 0;
                    text_box_height = font_size as f32 * 2.5;
                    let mut y = (h as f32 * 3.0 / 11.0).max(200.0);
                    let step = (h as f32 - y) / 8.0;
                    titolo_rect = rrect((w as f32 - text_box_width)/2.0, y, text_box_width, text_box_height);
                    y += step;
                    branca_rect = rrect((w as f32 - text_box_width)/2.0, y, text_box_width, text_box_height );
                    y += step;
                    giorno_rect = rrect((w as f32 - text_box_width * 4.0/4.0)/2.0, y, text_box_width/4.0, text_box_height );
                    mese_rect   = rrect((w as f32 - text_box_width/4.0)/2.0, y, text_box_width/4.0, text_box_height );
                    anno_rect   = rrect((w as f32 - text_box_width*-2.0/4.0)/2.0, y, text_box_width/4.0, text_box_height );
                    y += step;
                    server_rect = rrect((w as f32 - text_box_width)/2.0, y, text_box_width, text_box_height );
                    y += step;
                    utente_rect = rrect((w as f32 - text_box_width)/2.0, y, text_box_width, text_box_height );
                    y += step;
                    pw_rect     = rrect((w as f32 - text_box_width)/2.0, y, text_box_width, text_box_height );

                    y += step;
                    hd_rect     = rrect((w as f32 - text_box_width)/2.0 + text_box_height*0.3, y + text_box_height*0.3, text_box_height * 0.4, text_box_height * 0.4 );

                    gui_text_input_update(&mut rl, &mut idx, &mut text_box_active, &mut titolo_buf, 32, titolo_rect);
                    gui_text_input_update(&mut rl, &mut idx, &mut text_box_active, &mut branca_buf, 8, branca_rect);
                    gui_number_input_update(&mut rl, &mut idx, &mut text_box_active, &mut giorno_buf, 2, giorno_rect);
                    gui_number_input_update(&mut rl, &mut idx, &mut text_box_active, &mut mese_buf, 2, mese_rect);
                    gui_number_input_update(&mut rl, &mut idx, &mut text_box_active, &mut anno_buf, 2, anno_rect);
                    gui_text_input_update(&mut rl, &mut idx, &mut text_box_active, &mut server_buf, 32, server_rect);
                    gui_text_input_update(&mut rl, &mut idx, &mut text_box_active, &mut utente_buf, 32, utente_rect);
                    gui_text_input_update(&mut rl, &mut idx, &mut text_box_active, &mut pw_buf, 32, pw_rect);
                    gui_check_box_update(&mut rl, &mut idx, &mut text_box_active, hd_rect, &mut hd_images);
                },
                AppTab::SelectionLab => {
                    if last_image_loaded {
                        thread::sleep(Duration::from_millis(500));
                        last_image_loaded = false;
                    }

                    if let Some(path) = file_queue.pop_front() {
                        if file_queue.len() == 0 {
                            last_image_loaded = true;
                        }
                        
                        println!("[INFO]: Loading image: `{}`...", path.display());
                        if let Ok(img_) = ImageReader::open(path.clone()) {
                            if let Ok(img) = img_.decode() {
                                let img_scaled;
                                let size = img.dimensions();

                                let small_dim = if hd_images { HD_SMALLER_DIMENSION } else { SMALLER_DIMENSION };
                                let big_dim = if hd_images { HD_BIGGER_DIMENSION } else { BIGGER_DIMENSION };

                                if size.0 > size.1 {
                                    img_scaled = img.resize_to_fill(big_dim, small_dim, Lanczos3);
                                } else {
                                    img_scaled = img.resize_to_fill(small_dim, big_dim, Lanczos3);
                                }

                                let bytes_ = img_scaled.to_rgb8();
                                let mut bytes = bytes_.as_raw().to_owned();
                                
                                let rimg = unsafe {
                                    Image::from_raw(raylib::ffi::Image {
                                        data: bytes.as_mut_ptr() as *mut c_void,
                                        width: img_scaled.width() as i32,
                                        height: img_scaled.height() as i32,
                                        mipmaps: 1,
                                        format: PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8 as i32
                                    })
                                };
                                
                                // not eliminating unwrap because do not want to mess with mem::forget
                                // should work fine anyway...
                                let texture = rl.load_texture_from_image(&thread, &rimg).unwrap();
                                std::mem::forget(rimg);
                                let filename = path.file_name().unwrap_or_default().to_str().unwrap_or_default().to_owned();

                                images.push(ImgData::new(path.canonicalize().unwrap_or(path), filename, img_scaled, texture));
                            }
                        }
                    } else {
                        if check_ctrl_shortcut(&rl, Some(KeyboardKey::KEY_TAB)) {
                            next_tab = get_next_tab(app_tab);
                            text_box_active = -1;
                        }

                        if rl.is_key_pressed(KeyboardKey::KEY_DELETE) {
                            images.remove(file_list_active as usize);
                            list_moved_by_key = true;
                        }

                        let prev_file_list_active = file_list_active;

                        let fast_step = (images.len() as f32 / 10.0).ceil() as i32;
                        if check_ctrl_shortcut(&rl, None) {
                            if rl.is_key_pressed(KeyboardKey::KEY_UP) || is_key_pressed_repeat(KeyboardKey::KEY_UP) {
                                file_list_active -= fast_step;
                                list_moved_by_key = true;
                            }
                            if rl.is_key_pressed(KeyboardKey::KEY_DOWN) || is_key_pressed_repeat(KeyboardKey::KEY_DOWN) {
                                file_list_active += fast_step;
                                list_moved_by_key = true;
                            }
                        } else {
                            if rl.is_key_pressed(KeyboardKey::KEY_UP) || is_key_pressed_repeat(KeyboardKey::KEY_UP) {
                                file_list_active -= 1;
                                list_moved_by_key = true;
                            }
                            if rl.is_key_pressed(KeyboardKey::KEY_DOWN) || is_key_pressed_repeat(KeyboardKey::KEY_DOWN) {
                                file_list_active += 1;
                                list_moved_by_key = true;
                            }
                        }

                        file_list_active = file_list_active.min(images.len() as i32 - 1).max(0);

                        if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) || rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT) {
                            if (file_list_active - prev_file_list_active).abs() <= 1 {
                                images.swap(prev_file_list_active as usize, file_list_active as usize);
                            } else {
                                let img_to_move = images.remove(prev_file_list_active as usize);
                                images.insert(file_list_active as usize, img_to_move);
                            }
                        }


                        if rl.is_key_pressed(KeyboardKey::KEY_R) {
                            if file_list_active >= 0 && !images.is_empty() {
                                let rotated_image = if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) || rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT) {
                                    images[file_list_active as usize].image.rotate270()
                                } else {
                                    images[file_list_active as usize].image.rotate90()
                                };

                                let bytes_ = rotated_image.to_rgb8();
                                let mut bytes = bytes_.as_raw().to_owned();
                                
                                let rimg = unsafe{
                                    Image::from_raw(raylib::ffi::Image {
                                        data: bytes.as_mut_ptr() as *mut c_void,
                                        width: rotated_image.width() as i32,
                                        height: rotated_image.height() as i32,
                                        mipmaps: 1,
                                        format: PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8 as i32
                                    })
                                };
                                
                                // not eliminating unwrap because do not want to mess with mem::forget
                                // should work fine anyway...
                                let texture = rl.load_texture_from_image(&thread, &rimg).unwrap();
                                std::mem::forget(rimg);
                                images[file_list_active as usize].image = rotated_image;
                                images[file_list_active as usize].texture = texture;
                            }
                        }
                    }
                }
            };
        } else {
            match upload_status {
                UploadStatus::None => {
                    titolo = clean_string(String::from_utf8(titolo_buf.clone()).unwrap_or_default());
                    branca = clean_string(String::from_utf8(branca_buf.clone()).unwrap_or_default()).to_uppercase();
                    
                    data = format!("{:0>2}{:0>2}{:0>2}",String::from_utf8(anno_buf.clone()).unwrap_or_default(), String::from_utf8(mese_buf.clone()).unwrap_or_default(), String::from_utf8(giorno_buf.clone()).unwrap_or_default());

                    anno = 2000 + String::from_utf8(anno_buf.clone()).unwrap_or_default().parse::<usize>().unwrap_or(0);
                    mese = String::from_utf8(mese_buf.clone()).unwrap_or_default().parse::<usize>().unwrap_or(0);

                    server = String::from_utf8(server_buf.clone()).unwrap_or_default();
                    utente = String::from_utf8(utente_buf.clone()).unwrap_or_default();
                    password = String::from_utf8(pw_buf.clone()).unwrap_or_default();

                    image_dir = format!("{}_{}_{}", data, branca, titolo);
                    // println!("{}", image_dir);
                    upload_status = UploadStatus::CreatingDir;
                },
                UploadStatus::CreatingDir => {
                    let _ = fs::remove_dir_all(image_dir.clone());
                    upload_status = match fs::create_dir(image_dir.clone()) {
                        Ok(_) => UploadStatus::SavingImage(0),
                        Err(e) => {
                            eprintln!("[ERROR]: Impossibile creare la cartella `{}`: {}", image_dir, e);
                            UploadStatus::Error(format!("Impossibile creare la cartella `{}`.", image_dir)) 
                        },
                    };
                },
                UploadStatus::SavingImage(i) => {
                    let new_name = format!("{}/{}_{}_{}_{:03}.JPG", image_dir, data, branca, titolo, i + 1);

                    upload_status = match images[i].image.save(new_name) {
                        Ok(_) => {
                            if i+1 < images.len() {
                                 UploadStatus::SavingImage(i+1)
                            } else {
                                files_to_upload = find_files(&image_dir);
                                files_to_upload.reverse();
                                UploadStatus::DoneSaving
                            }
                        },
                        Err(e) => {
                            eprintln!("[ERROR]: Impossibile salvare le immagini: {}", e);
                            UploadStatus::Error(String::from("Impossibile salvare le immagini"))
                        }
                    };
                    
                },
                UploadStatus::DoneSaving => {},
                UploadStatus::Connecting => {
                    upload_status = match FtpStream::connect(format!("{}:21", server)) {
                        Ok(mut s) => {
                            match s.login(&utente, &password) {
                                Ok(_) => {
                                    ftp_stream = Some(s); 
                                    UploadStatus::UploadingImage(0)
                                },
                                Err(e) => {
                                    eprintln!("[ERROR]: Impossibile autenticarsi in `{}` (utente: {}): {}", server, utente, e);
                                    UploadStatus::Error(format!("Impossibile autenticarsi in `{}` (utente: `{}`)", server, utente))
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("[ERROR]: Impossibile connettersi a `{}`: {}", server, e);
                            UploadStatus::Error(format!("Impossibile connettersi a `{}`", server))
                        }
                    };
                },
                UploadStatus::UploadingImage(i) => {
                    if let Some(stream) = &mut ftp_stream {
                        if i == 0 {
                            // println!("{:?}", stream.list(None).unwrap());
                            let dir = if mese < 8 {
                                format!("{}-{}", anno - 1, anno)
                            } else {
                                format!("{}-{}", anno, anno + 1)
                            };

                            if let Err(e) = stream.cwd(&dir) {
                                eprintln!("[ERROR]: Sul server ftp `{}` non esiste la cartella `{}`: {}\nImpossibile caricare le immagini.", server, dir, e);
                                upload_status = UploadStatus::Error(format!("Sul server ftp `{}` non esiste la cartella `{}`.\nImpossibile caricare le immagini.", server, dir));
                            } else {
                                println!("[FTP]: cd {}/", dir);
                                if let Err(e) = stream.mkdir(&image_dir) {
                                    eprintln!("[ERROR]: Sul server ftp `{}` esiste già la cartella `{}`: {}\nImpossibile caricare le immagini.", server, image_dir, e);
                                    upload_status = UploadStatus::Error(format!("Sul server ftp `{}` esiste già la cartella `{}`.\nImpossibile caricare le immagini.", server, image_dir));
                                }
                            }
                            // IDK, FTP error... at this point let it just crash
                            stream.transfer_type(ftp::types::FileType::Image).unwrap();
                        }
                        
                        match upload_status {
                            UploadStatus::Error(_) => {},
                            UploadStatus::UploadingImage(i) => {
                                let file = files_to_upload[i].clone();
                                let content = fs::read(file.clone()).unwrap();
                                let mut reader = Cursor::new(content);
                                stream.put(&file.to_slash().unwrap(), &mut reader).unwrap();

                                upload_status = UploadStatus::UploadingImage(i+1);

                                if i >= files_to_upload.len() - 1 {
                                    // Don't care...
                                    let _ = stream.quit();
                                    upload_status = UploadStatus::Done;
                                }
                            },
                            _ => {},
                        }
    
                    }

                },
                UploadStatus::Done => {},
                UploadStatus::Error(_) => {},
            };
        }
        
        let mut d = rl.begin_drawing(&thread);
        (w, h) = (d.get_screen_width(), d.get_screen_height());
        font_size = h/42;
        // d.gui_set_style(GuiControl::DEFAULT, GuiControlProperty::BASE_COLOR_NORMAL as i32, GUI_NORMAL_COLOR.color_to_int());
        // d.gui_set_style(GuiControl::DEFAULT, GuiControlProperty::BASE_COLOR_FOCUSED as i32, GUI_FOCUSED_COLOR.color_to_int());
        // d.gui_set_style(GuiControl::DEFAULT, GuiControlProperty::BASE_COLOR_PRESSED as i32, GUI_PRESSED_COLOR.color_to_int());
        d.gui_set_style(GuiControl::DEFAULT, GuiDefaultProperty::TEXT_SIZE as i32, font_size);
        d.gui_set_style(GuiControl::DEFAULT, GuiDefaultProperty::BACKGROUND_COLOR as i32, BACKGROUND_COLOR.color_to_int());
        let item_height = ((h * 4 / 5) - d.gui_get_style(GuiControl::LISTVIEW, GuiListViewProperty::LIST_ITEMS_SPACING as i32) * 7) / 7;
        d.gui_set_style(GuiControl::LISTVIEW, GuiListViewProperty::LIST_ITEMS_HEIGHT as i32, item_height);

        d.clear_background(BACKGROUND_COLOR);

        // RENDERING
        let title = "FOTO TPM";
        let title_width = d.measure_text(title, font_size * 3); // drawn at the end so it appears above everything
        if !upload {
            match app_tab {
                AppTab::InputData => {
                    if let Some(tab) = draw_tab_buttons(&mut d, app_tab, w as f32, h as f32, font_size) {
                        next_tab = tab;
                    }
                    let mut idx = 0;
                    gui_text_input(&mut d, &mut idx, text_box_active, "Titolo dell'attività", &mut titolo_buf, font_size, titolo_rect);
                    gui_text_input(&mut d, &mut idx, text_box_active, "Branca", &mut branca_buf, font_size, branca_rect);
                    gui_text_input(&mut d, &mut idx, text_box_active, "Giorno", &mut giorno_buf, font_size, giorno_rect);
                    gui_text_input(&mut d, &mut idx, text_box_active, "Mese", &mut mese_buf, font_size, mese_rect);
                    gui_text_input(&mut d, &mut idx, text_box_active, "Anno", &mut anno_buf, font_size, anno_rect);
                    gui_text_input(&mut d, &mut idx, text_box_active, "Server", &mut server_buf, font_size, server_rect);
                    gui_text_input(&mut d, &mut idx, text_box_active, "Utente", &mut utente_buf, font_size, utente_rect);
                    gui_seecret_text_input(&mut d, &mut idx, text_box_active, "Password", &mut pw_buf, font_size, pw_rect);

                    // let hd_text = CString::new("HD (prima di caricare le foto)").unwrap();

                    // d.gui_check_box(hd_rect, None, &mut hd_images);
                    gui_check_box(&mut d, &mut idx, text_box_active, hd_rect,  hd_images);

                    let hd_color = if hd_images { Color::WHITE } else { Color::GRAY };

                    let hd_text = "HD ";
                    let hd_text_size = d.measure_text(&hd_text, font_size);

                    d.draw_text(hd_text, (hd_rect.x + hd_rect.width * 2.0) as i32, (hd_rect.y + hd_rect.height) as i32 - font_size, font_size, hd_color);
                    let small_font_size = font_size * 3 / 4;
                    d.draw_text("(premere prima di importare le foto)", (hd_rect.x + hd_rect.width * 2.0) as i32 + hd_text_size, (hd_rect.y + hd_rect.height) as i32 - small_font_size, small_font_size, hd_color);
                    
                    if file_queue.len() > 0 {
                        let small_font_size = font_size;
                        let text = format!("{} fotografie in attesa di caricamento", file_queue.len());
                        let text_size = d.measure_text(&text, small_font_size);
                        d.draw_text(&text, (w - text_size)/2, h-small_font_size-15, small_font_size, Color::WHITE);
                    }
                    
                    let version_font_size = font_size * 9 / 10;
                    let version_text_size = d.measure_text(&version_text, version_font_size);
                    d.draw_text(&version_text, w-version_text_size - 10, h-version_font_size - 5, version_font_size, Color::WHITE.alpha(0.5));
                },
                AppTab::SelectionLab => {
                    if images.is_empty() {
                        if let Some(tab) = draw_tab_buttons(&mut d, app_tab, w as f32, h as f32, font_size) {
                            next_tab = tab;
                        }

                        let drop_text = "Rilasci le foto";
                        let drop_text_width = d.measure_text(drop_text, font_size*2);
                        d.draw_text(drop_text, (w-drop_text_width)/2, h*3/7, font_size*2, Color::WHITE);

                        let version_font_size = font_size * 9 / 10;
                        let version_text_size = d.measure_text(&version_text, version_font_size);
                        d.draw_text(&version_text, w-version_text_size - 10, h-version_font_size - 5, version_font_size, Color::WHITE.alpha(0.5));
                    } else if !file_queue.is_empty() || last_image_loaded {
                        let load_text = format!("Caricando {} foto{}", file_queue.len(), match (d.get_time() as u32) % 4 {
                            0 => "",
                            1 => ".",
                            2 => "..",
                            3 => "...",
                            _ => unreachable!()
                        });
                        let load_text_width = d.measure_text(load_text.as_str(), font_size*2);
                        let img_w = images.last().unwrap().image.width() as f32;
                        let img_h = images.last().unwrap().image.height() as f32;
                        let scale_x = w as f32 /img_w;
                        let scale_y = h as f32 /img_h;
                        let scale = scale_x.max(scale_y);
                        d.draw_texture_ex(&images.last().unwrap().texture, rvec2(w as f32 / 2.0 - scale * img_w * 0.5, h as f32 / 2.0 - scale * img_h * 0.5), 0.0, scale, Color::WHITE.alpha(0.5));
                        d.draw_text(&load_text, (w-load_text_width)/2, h*3/7, font_size*2, Color::WHITE);
                    } else {
                        let upload_text = "upload";
                        let upload_button_width = d.measure_text(&upload_text, font_size) as f32 + 20.0 * 2.0;
                        let upload_button_height = font_size as f32 + 20.0 * 2.0;
                        let upload_button_rect = rrect(
                            w as f32 - upload_button_width - font_size as f32, 
                            h as f32 - upload_button_height - font_size as f32,
                            upload_button_width, upload_button_height
                        );

                        let img_w = images[file_list_active as usize].image.width() as f32;
                        let img_h = images[file_list_active as usize].image.height() as f32;
                        let scale_x = (w as f32 * 4.0/5.0)/img_w;
                        let scale_y = (h as f32 * 4.0/5.0)/img_h;
                        let scale = scale_x.min(scale_y);

                        let img_x = w as f32 * (2.0 + 3.0) / 8.0 - (img_w * scale) / 2.0 - (w as f32 - upload_button_rect.x) / 2.0;
                        let img_y = (h as f32 / 5.0).max(167.0);
                        d.draw_texture_ex(&images[file_list_active as usize].texture, rvec2(img_x, img_y), 0.0, scale, Color::WHITE);

                        if let Some(tab) = draw_tab_buttons(&mut d, app_tab, w as f32, h as f32, font_size) {
                            next_tab = tab;
                        }


                        let upload_text_cstr = CString::new(upload_text).unwrap_or_default();
                        let inputs_vec = vec![&titolo_buf, &branca_buf, &giorno_buf, &mese_buf, &anno_buf, &server_buf, &utente_buf, &pw_buf];
                        let input_not_given = inputs_vec.iter().any(|x| x.is_empty());
                        let upload_pressed = d.gui_button(upload_button_rect, Some(upload_text_cstr.as_c_str()));

                        if upload_pressed {
                            upload = true;

                            if input_not_given {
                                text_box_active = -1;
                                for b in inputs_vec.iter() {
                                    text_box_active += 1;
                                    if b.is_empty() {
                                        break;
                                    }
                                }
                                upload_status = UploadStatus::Error(format!("Alcune voci nella scheda `{}` non sono state compilate.", AppTab::InputData));
                            }
                        }
                        
                        {
                            let load_text = format!("{}/{}", file_list_active+1, images.len());
                            let load_text_width = d.measure_text(load_text.as_str(), font_size);
                            draw_outlined_text(&mut d, load_text.as_str(), w*5/8 - load_text_width/2, h-font_size, font_size, 2, Color::WHITE, Color::BLACK);
                        }

                        let item_height = d.gui_get_style(GuiControl::LISTVIEW, GuiListViewProperty::LIST_ITEMS_HEIGHT as i32) + d.gui_get_style(GuiControl::LISTVIEW, GuiListViewProperty::LIST_ITEMS_SPACING as i32);
                        let max_viewable_index_offset = (h * 4 / 5) / item_height;
                        if list_moved_by_key {
                            while file_list_scroll_index + max_viewable_index_offset - 1 <= file_list_active && file_list_scroll_index < images.len() as i32 {
                                file_list_scroll_index += 1;
                            }
                            while file_list_scroll_index + 1 > file_list_active && file_list_scroll_index > 0 {
                                file_list_scroll_index -= 1;
                            }
                            list_moved_by_key = false;
                        }

                        let list_rect = rrect(0.0, (h as f32 / 5.0).max(167.0), w as f32/6.0, (h as f32 * 4.0 / 5.0).min(h as f32-167.0));

                        if d.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                            let mouse_in_boundaries = unsafe { CheckCollisionPointRec(d.get_mouse_position().into(), list_rect.into())};
                            if mouse_in_boundaries {
                                let idx = ((d.get_mouse_y() as f32 - list_rect.y) / item_height as f32).floor();
                                if idx >= 0.0 {
                                    file_list_active = file_list_scroll_index + idx as i32;
                                }
                            }
                        }

                        let list_text = images.iter()
                            .skip(file_list_scroll_index as usize).take(max_viewable_index_offset as usize)
                            .map(|a| a.filename.clone()).collect::<Vec<_>>().join("\n");
                        let list_cstr_text = CString::new(list_text).unwrap_or_default();
                        // let list_cstr = CString::new(list_text).unwrap_or_default();
                        
                        let mut scroll_idx = 0;
                        let preview_width = w / 4;
                        d.draw_rectangle(list_rect.x as i32, list_rect.y as i32, preview_width, list_rect.height as i32, Color::GRAY);
                        let mut idx = file_list_active - file_list_scroll_index;
                        d.gui_list_view(list_rect, Some(list_cstr_text.as_c_str()), &mut scroll_idx, &mut idx);
                        file_list_active = file_list_scroll_index + idx;
                        // println!("active: {} | scroll: {}",file_list_active,file_list_scroll_index);

                        
                        for (i, img) in images.iter().skip(file_list_scroll_index as usize).take(max_viewable_index_offset as usize).enumerate() {
                            let max_w = preview_width as f32 - list_rect.width;
                            let max_h = item_height as f32;
                            let img_w = img.image.width() as f32;
                            let img_h = img.image.height() as f32;
                            let scale_x = max_w /img_w;
                            let scale_y = max_h /img_h;
                            let mut scale = scale_x.min(scale_y);

                            let mut y = (h as f32 / 5.0).max(167.0) + i as f32 * item_height as f32;
                            let mut x = list_rect.width;

                            let mut color_fade = 1.0;
                            
                            if i as i32 != file_list_active - file_list_scroll_index {
                                scale *= 0.85;
                                color_fade *= 0.9;
                            }

                            y += (max_h - img_h * scale) / 2.0;
                            x += (max_w - img_w * scale) / 2.0;
                            
                            d.draw_texture_ex(&img.texture, rvec2(x, y), 0.0, scale, Color::WHITE);
                            
                            let num_text = format!("{}", i+file_list_scroll_index as usize + 1);
                            let outline_size = 2;
                            draw_outlined_text(&mut d, &num_text, x as i32 + outline_size * 2, y as i32 + outline_size + 1, font_size, outline_size, Color::WHITE.alpha(color_fade), Color::BLACK.alpha(color_fade/2.0));
                        }
                        
                        file_list_scroll_index += scroll_idx;
                        
                    }
                }
            };
        } else {
            match upload_status {
                UploadStatus::None => {},
                UploadStatus::CreatingDir => {
                    let upload_text = "Creazione della cartella";
                    let upload_label_text = format!("{} `{}`{}", upload_text, image_dir, match (d.get_time() as u32) % 4 {
                        0 => "",
                        1 => ".",
                        2 => "..",
                        3 => "...",
                        _ => unreachable!()
                    });
                    let upload_text_width = d.measure_text(upload_label_text.as_str(), font_size*2);
                    
                    d.draw_text(upload_label_text.as_str(), (w-upload_text_width)/2, h*3/7, font_size*2, Color::WHITE);
                },
                UploadStatus::SavingImage(i) => {
                    let upload_text = "Salvando le immagini in";
                    let upload_label_text = format!("{} `{}`{}", upload_text, image_dir, match (d.get_time() as u32) % 4 {
                        0 => "",
                        1 => ".",
                        2 => "..",
                        3 => "...",
                        _ => unreachable!()
                    });
                    let upload_text_width = d.measure_text(upload_label_text.as_str(), font_size*2);
                    
                    d.draw_text(upload_label_text.as_str(), (w-upload_text_width)/2, h*3/7, font_size*2, Color::WHITE);

                    let progress_bar_width = w as f32 / 3.0;
                    d.gui_progress_bar(rrect((w as f32 - progress_bar_width) / 2.0, h as f32 * 0.5, progress_bar_width, 25.0), None, None, &mut (i as f32), 0.0, (images.len()-1) as f32);
                },
                UploadStatus::DoneSaving => {
                    let upload_button_width = 550.0;
                    let upload_button_height = font_size as f32*2.0;
                    let upload_text = CString::new(format!("Caricare le foto sul server")).unwrap_or_default();
                    if d.gui_button(rrect((w as f32 - upload_button_width) / 2.0, (h as f32 - upload_button_height)/2.0, upload_button_width, upload_button_height ), Some(upload_text.as_c_str())) {
                        upload_status = UploadStatus::Connecting;
                    }
                },
                UploadStatus::Connecting => {
                    let upload_text = "Connessione a";
                    let upload_label_text = format!("{} `{}`{}", upload_text, server, match (d.get_time() as u32) % 4 {
                        0 => "",
                        1 => ".",
                        2 => "..",
                        3 => "...",
                        _ => unreachable!()
                    });
                    let upload_text_width = d.measure_text(upload_label_text.as_str(), font_size*2);
                    
                    d.draw_text(upload_label_text.as_str(), (w-upload_text_width)/2, h*3/7, font_size*2, Color::WHITE);
                },
                UploadStatus::UploadingImage(i) => {
                    let upload_text = "Uploading";
                    let upload_label_text = format!("{}{}", upload_text, match (d.get_time() as u32) % 4 {
                        0 => "",
                        1 => ".",
                        2 => "..",
                        3 => "...",
                        _ => unreachable!()
                    });
                    let upload_text_width = d.measure_text(upload_text, font_size*2);
                    
                    d.draw_text(upload_label_text.as_str(), (w-upload_text_width)/2, h*3/7, font_size*2, Color::WHITE);

                    let progress_bar_width = w as f32 / 3.0;
                    d.gui_progress_bar(rrect((w as f32 - progress_bar_width) / 2.0, h as f32 * 0.5, progress_bar_width, 25.0), None, None, &mut (i as f32), 0.0, (files_to_upload.len()-1) as f32);
                },
                UploadStatus::Error(ref e) => {
                    let error_text_width = d.measure_text(e.as_str(), font_size);
                    d.draw_text(e.as_str(), (w-error_text_width)/2, h*3/7, font_size, Color::RED);

                    let back_button_width = 500.0;
                    let back_button_height = font_size as f32*2.0;
                    let back_text = CString::new("Indietro").unwrap_or_default();
                    if d.gui_button(rrect((w as f32 - back_button_width) / 2.0, h as f32 * 3.0/7.0 + font_size as f32*3.0 + back_button_height/2.0, back_button_width, back_button_height ), Some(back_text.as_c_str())) {
                        upload = false;
                        upload_status = UploadStatus::None;
                        next_tab = AppTab::InputData;
                    }
                },
                UploadStatus::Done => {
                    let done_text = "Fatto!";
                    let done_text_width = d.measure_text(done_text, font_size*2);
                    d.draw_text(done_text, (w-done_text_width)/2, h*3/7, font_size*2, THEME_COLOR);

                    let close_text = "adesso l'applicazione può essere chiusa";
                    let close_text_width = d.measure_text(close_text, font_size);
                    d.draw_text(close_text, (w-close_text_width)/2, h*3/7 + font_size*3, font_size, THEME_COLOR);
                }
            };

            
        }
        // d.draw_text(title, (w-title_width)/2, 25, font_size * 3, THEME_COLOR);
        draw_outlined_text(&mut d, title, (w-title_width)/2, 25, font_size*3, font_size/10, THEME_COLOR, THEME_COLOR);
    }

    if !images.is_empty() {
        let date = Local::now();
        let file_list_path = format!("fototpm-imglist_{}.txt", date.format("%Y-%m-%d %H:%M:%S"));
        let _ = save_used_files(&file_list_path, &images);
        if let Ok(full_path) = PathBuf::from(file_list_path).canonicalize() {
            println!("[INFO]: List of image files in use saved in `{}`.", full_path.display());
        }
    }

}

fn main() {
    gui_app()
}
