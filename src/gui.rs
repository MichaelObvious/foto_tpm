use std::ffi::{CStr, CString};

use raylib::ffi::CheckCollisionPointRec;
use raylib::prelude::*;

pub fn gui_text_input_update(rl: &mut RaylibHandle, idx: &mut i32, active_index: &mut i32, buffer: &mut Vec<u8>, max_len: usize, text_box: Rectangle) {

    let mouse_pressed = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
    let mouse_in_boundaries = unsafe { CheckCollisionPointRec(rl.get_mouse_position().into(), text_box.into()) };
    if mouse_pressed {
        if mouse_in_boundaries {
            *active_index = *idx;
        } else if *active_index == *idx {
            *active_index = -1;
        }
    }
    if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
        *active_index = -1;
    }

    if idx == active_index {
        // update text
        // rl.set_mouse_cursor(MouseCursor::MOUSE_CURSOR_IBEAM);
        
        while let Some(c) = rl.get_char_pressed() {
            let c = c as u8;
            // println!("`{}`", c);
            if (c >= 32) && (c <= 125) && (buffer.len() < max_len) {
                // println!("{:?}", buffer);
                buffer.push(c);
                // println!("add char {:?}", buffer);
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_V) && (rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) || rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL)) {
            if let Ok(x) = rl.get_clipboard_text() {
                for c in x.chars() {
                    let c = c as u8;
                    // println!("`{}`", c);
                    if (c >= 32) && (c <= 125) && (buffer.len() < max_len) {
                        // println!("{:?}", buffer);
                        buffer.push(c);
                        // println!("add char {:?}", buffer);
                    }
                }
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE) {
            // println!("{:?}", buffer);
            if rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT) || rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
                buffer.clear();
            }
            buffer.pop();
            // println!("delete   {:?}", buffer);
            // println!("BACK!");
        }
    }

    *idx += 1;
}

pub fn gui_number_input_update(rl: &mut RaylibHandle, idx: &mut i32, active_index: &mut i32, buffer: &mut Vec<u8>, max_len: usize, text_box: Rectangle) {
    let mouse_pressed = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
    let mouse_in_boundaries = unsafe { CheckCollisionPointRec(rl.get_mouse_position().into(), text_box.into()) };
    if mouse_pressed {
        if mouse_in_boundaries {
            *active_index = *idx;
        } else if *active_index == *idx {
            *active_index = -1;
        }
    }
    if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
        *active_index = -1;
    }

    if idx == active_index {
        // update text
        // rl.set_mouse_cursor(MouseCursor::MOUSE_CURSOR_IBEAM);
        
        while let Some(c) = rl.get_char_pressed() {
            let c = c as u8;
            // println!("`{}`", c);
            if (c >= 48) && (c <= 57) && (buffer.len() < max_len) {
                buffer.push(c);
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE) {
            if rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT) || rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
                buffer.clear();
            }
            buffer.pop();
            // println!("BACK!");
        }
    }
    *idx += 1;
}

pub fn gui_text_input(d: &mut RaylibDrawHandle, idx: &mut i32, active_idx: i32, label: &str, buffer: &mut Vec<u8>, size: i32, text_box: Rectangle) {
    let is_active = *idx == active_idx;
    
    let mut buf = buffer.clone();
    buf.push('~' as u8);
    buf.push(0);

    d.draw_text(label, text_box.x as i32, text_box.y as i32 - size, size, Color::WHITE);
    let (bg_color, fg_color) =  if *idx == active_idx {
        (Color::LIGHTBLUE, Color::DARKCYAN)
    } else {
        (Color::GRAY, Color::BLACK)
    };

    let mut str = unsafe { CStr::from_bytes_with_nul_unchecked(&buf).to_str().unwrap_or_default().to_owned() };
    let mut s = str.clone();
    let side_padding = size*2/3;
    
    while d.measure_text(&s, size) > text_box.width as i32 - side_padding*2 {
        let mut chars = str.chars();
        if is_active {
            chars.next();
            s = format!("...{}", chars.as_str());
        } else {
            chars.next_back();
            s = format!("{}...", chars.as_str());
        }
        str = chars.as_str().to_owned();
    }

    s = if is_active && (d.get_time() / 0.5)as u32 % 2 == 0 {
        s.replace('~', "_")
    } else {
        s.replace('~', " ")
    };

    let outline_size = 3;
    d.draw_rectangle(text_box.x as i32, text_box.y as i32, text_box.width as i32, text_box.height as i32, fg_color);
    d.draw_rectangle(text_box.x as i32 + outline_size, text_box.y as i32 + outline_size, text_box.width as i32 - outline_size*2, text_box.height as i32 - outline_size*2, bg_color);
    d.draw_text(&s, text_box.x as i32 + side_padding, text_box.y as i32 + (text_box.height * 0.6) as i32 - size / 2, size, fg_color);
    // d.gui_text_box(text_box, &mut buf, *idx == active_idx);
    *idx += 1;
}

pub fn gui_seecret_text_input(d: &mut RaylibDrawHandle, idx: &mut i32, active_idx: i32, label: &str, buffer: &mut Vec<u8>, size: i32, text_box: Rectangle) {
    let mut seecret_data = buffer.iter().map(|_| '*' as u8).collect::<Vec<_>>();

    gui_text_input(d, idx, active_idx, label, &mut seecret_data, size, text_box);
}

// pub fn main() {
//     let (mut rl, thread) = raylib::init()
//         .size(640, 480)
//         .title("Foto TPM")
//         .resizable()
//         .build();

//     let files = vec!["/home/obvious/Pictures/sas/lupetti-1.jpg", "/home/obvious/Pictures/sas/lupetti-2.jpg", "/home/obvious/Pictures/sas/lupetti-3.JPG", "/home/obvious/Pictures/sas/lupetti-4.JPG", "/home/obvious/Pictures/sas/lupetti-5.JPG"];
//     let mut images_data: Vec<Vec<u8>> = Vec::new();
//     let mut textures: Vec<Texture2D> = Vec::new();

//     {
//         let times = 1;
//         for i in 0..times {
//             for path in files.iter() {
//                 let img = ImageReader::open(path).unwrap().decode().unwrap();
//                 let img_scaled;
//                 let size = img.dimensions();
//                 if size.0 > size.1 {
//                     img_scaled = img.resize_to_fill(800, 600, Triangle);
//                 } else {
//                     img_scaled = img.resize_to_fill(600, 800, Triangle);
//                 }
//                 images_data.push(img_scaled.as_rgb8().unwrap().as_raw().to_owned());
//                 let rimg = unsafe{
//                     Image::from_raw(raylib::ffi::Image {
//                         data: images_data.last_mut().unwrap().as_mut_ptr() as *mut c_void,
//                         width: img_scaled.width() as i32,
//                         height: img_scaled.height() as i32,
//                         mipmaps: 1,
//                         format: PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8 as i32
//                     })
//                 };

//                 textures.push(rl.load_texture_from_image(&thread, &rimg).unwrap());
//             }
//             println!("IMAGES = {:02}/{:02}", (i+1)*files.len(), times*files.len());
//         }
//     }

    
//     // let images = files.iter().map(|x| rl.load_texture(&thread, x).unwrap()).collect::<Vec<_>>();
    
//     let mut index = 0;
//     let mut text = Vec::with_capacity(256);
//     text.push(0);
//     // text.fill(0);

//     let mut eendecs = -1;
    
//     while !rl.window_should_close() {
        
//         let mut text_input_idx = 0;
//         gui_text_input_update(&mut rl, &mut text_input_idx, &mut eendecs, &mut text, 16, Rectangle { x:400.0, y:200.0, width: 200.0, height: 50.0 });

//         let mut d = rl.begin_drawing(&thread);
        

//         if d.is_key_pressed(KeyboardKey::KEY_LEFT_ALT) {
//             index = (index + 1) % textures.len();
//         }

        
//         let (w, h) = (d.get_screen_width(), d.get_screen_height());
//         let font_size = h/20;
//         d.gui_set_style(GuiControl::DEFAULT, GuiDefaultProperty::TEXT_SIZE as i32, h/42);
         
//         d.clear_background(Color::new(0x18, 0x18, 0x18, 0xff));

//         let title_font_size = h/20;
//         let title_text = "Hello, world!";
//         let title_width = measure_text(title_text, title_font_size);
//         d.draw_text(title_text, (w-title_width)/2, h/2, title_font_size, Color::WHITE);
//         // d.draw_texture(&images[index], 0, 0, Color::WHITE);
//         // d.draw_texture(&textures[index], 0, 0, Color::WHITE);

//         text_input_idx = 0;
//         // gui_seecret_text_input(&mut d, &mut text_input_idx, ac"yoo", &mut text, 20, Rectangle { x:400.0, y:200.0, width: 200.0, height: 50.0 })


//         // d.gui_text_box(Rectangle { x:0.0, y:0.0, width: 100.0, height: 200.0 }, &mut text, true);
//         // d.gui_text_input_box(Rectangle { x:0.0, y:0.0, width: 500.0, height: 200.0 }, None, None, None, &mut text, 100);
//         // unsafe{ffi::GuiTextInputBox(
//         //     Rectangle { x:0.0, y:0.0, width: 100.0, height: 200.0 }.into(),
//         //     std::ptr::null(),
//         //     std::ptr::null(),
//         //     std::ptr::null(),
//         //     text.as_mut_ptr() as *mut _,
//         //     text.len() as u32,
//         //     std::ptr::null()
//         // );}
//     }
// }

pub fn draw_outlined_text(d: &mut RaylibDrawHandle, text: &str, x: i32, y: i32, font_size: i32, outline_size: i32, color: Color, outline_color: Color) {
    d.draw_text(text, x - outline_size, y - outline_size, font_size, outline_color);
    d.draw_text(text, x + outline_size, y - outline_size, font_size, outline_color);
    d.draw_text(text, x, y - outline_size, font_size, outline_color);
    d.draw_text(text, x, y + outline_size, font_size, outline_color);
    d.draw_text(text, x - outline_size, y + outline_size, font_size, outline_color);
    d.draw_text(text, x + outline_size, y + outline_size, font_size, outline_color);
    d.draw_text(text, x - outline_size, y, font_size, outline_color);
    d.draw_text(text, x + outline_size, y, font_size, outline_color);
    d.draw_text(text, x, y, font_size, color);
}