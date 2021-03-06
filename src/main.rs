//========================================================================
// read - simple win32 text editor written in rust
//------------------------------------------------------------------------
// Copyright (c) 2018 Ji Wong Park <sailfish009@gmail.com>
//
// This software is provided 'as-is', without any express or implied
// warranty. In no event will the authors be held liable for any damages
// arising from the use of this software.
//
// Permission is granted to anyone to use this software for any purpose,
// including commercial applications, and to alter it and redistribute it
// freely, subject to the following restrictions:
//
// 1. The origin of this software must not be misrepresented; you must not
//    claim that you wrote the original software. If you use this software
//    in a product, an acknowledgment in the product documentation would
//    be appreciated but is not required.
//
// 2. Altered source versions must be plainly marked as such, and must not
//    be misrepresented as being the original software.
//
// 3. This notice may not be removed or altered from any source
//    distribution.
//
//========================================================================

// hide console window
// #![windows_subsystem = "windows"]

#[macro_use]
extern crate lazy_static;
extern crate winapi; 

const SX: i32 = 200; const SY: i32 = 200; const W:  i32 = 800; const H:  i32 = 600;
const R_A: u8 = 250; const G_A: u8 = 250; const B_A: u8 = 250;
const R_B: u8 = 0;   const G_B: u8 = 0;   const B_B: u8 = 0;

use winapi::um::shellapi as shell;
use winapi::um::winuser as user;     
use winapi::um::wingdi as gdi;
use winapi::shared::windef::{HWND, HMENU, HBRUSH, HICON, HFONT, HGDIOBJ, HBITMAP, HDC, RECT, POINT};        
use winapi::shared::minwindef::{HINSTANCE, INT, UINT, DWORD, WPARAM, LPARAM, LRESULT, LPVOID, MAX_PATH };        
use user::{WS_OVERLAPPEDWINDOW, WS_VISIBLE, WNDCLASSW, LPCREATESTRUCTW};        
use winapi::um::winnt::{LPCWSTR, LPWSTR, LONG};
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::ptr;
use std::string::String;
use std::sync::Mutex;
use std::io::Read;
use std::io::Write;
use std::fs::File;
// use std::io::prelude::*;
// use std::thread;
// use std::iter::Zip;

struct CH {x :LONG,  y :LONG, c :char}

lazy_static!
{
  // MODE:  0: save  1(i),2(a): edit  
  static ref MODE: Mutex<u8> = Mutex::new(0);
  static ref TEXT: Mutex<Vec<CH>> = Mutex::new(Vec::new());
  static ref POS: Mutex<POINT> = Mutex::new(POINT{x:0, y:0});
  static ref CHX: Mutex<LONG> = Mutex::new(0);
  static ref CHY: Mutex<LONG> = Mutex::new(0);
  static ref END: Mutex<u8> = Mutex::new(0);
}

// fn to_wchar(str : &str) -> *const u16 
// {
//   let v : Vec<u16> =
//     OsStr::new(str).encode_wide().chain(Some(0).into_iter()).collect();
//   v.as_ptr()
// }

fn to_wstring(str : &str) -> Vec<u16> 
{
  let v : Vec<u16> =
    OsStr::new(str).encode_wide().chain(Some(0).into_iter()).collect();
  v
}

// file
////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn fileio(w :HWND, f :HFONT, path :String, mode :u8)
{
  let mut result = File::open(path);
  match result
  {
    Ok(mut result) =>
    {
      match mode
      {
        // read 
        0 =>
        {
          hidecaret(w);
          clear();
          clearscreen(w);
          *POS.lock().unwrap() = POINT{x:0, y:0};
    
          let mut buffer = String::new();
          result.read_to_string(&mut buffer);
    
          for c in buffer.chars()
          {
            let x = {POS.lock().unwrap().x};
            let y = {POS.lock().unwrap().y};
            match c
            {
              '\r' =>
              {
                let ch = CH{x:x, y:y,c:c};
                save(ch);
                POS.lock().unwrap().x = 0;
                POS.lock().unwrap().y += 1;
              },
              '\n' =>{},
              _ => 
              { 
                let ch = CH{x:x,y:y,c:c};
                drawtext(w, f, ch, 0); 
                POS.lock().unwrap().x += 1;
              },
            }
          }
          showcaret(w);
          *END.lock().unwrap() = 1;
        },
        // write
        _ =>
        {
        },
      }
    },
    // file open failed.
    _ => 
    {
      println!("file error");
    },
  }

}

// gui
////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn clearscreen(w :HWND)
{
  // bug? call RedrawWindow() only should be work.
  // work around: call InvalidateRect() and call RedrawWindow()
  unsafe
  {
    user::InvalidateRect(w, ptr::null_mut(), 1);
    user::RedrawWindow(w, ptr::null_mut(), ptr::null_mut(), user::RDW_INVALIDATE | user::RDW_UPDATENOW);
  }
}
fn clearch(w :HWND, i :usize, r :i32)
{
  let rect = getrect(i);
  unsafe {user::InvalidateRect(w, &rect, r);}
}
fn clear() {let mut vec = {TEXT.lock().unwrap()}; vec.clear();}
fn remove(i :usize){let mut vec = {TEXT.lock().unwrap()}; vec.remove(i);}
fn insert(i :usize, c :CH){let mut vec = {TEXT.lock().unwrap()}; vec.insert(i, c)}
fn save(c :CH){let mut vec = {TEXT.lock().unwrap()}; vec.push(c);}
fn getlength() -> usize {let vec = {TEXT.lock().unwrap()}; let length = vec.len();length}
fn getx(i :usize) -> LONG {let vec = {TEXT.lock().unwrap()}; let x = vec[i].x; x}
fn gety(i :usize) -> LONG {let vec = {TEXT.lock().unwrap()}; let y = vec[i].y; y}
fn getc(i :usize) -> char {let vec = {TEXT.lock().unwrap()}; let c = vec[i].c; c}
fn setx(i :usize, x :LONG){let mut vec = {TEXT.lock().unwrap()}; vec[i].x = x;}
fn sety(i :usize, y :LONG){let mut vec = {TEXT.lock().unwrap()}; vec[i].y = y;}
fn getlastx() -> Option<usize>
{
  let y = {POS.lock().unwrap().y};
  let vec = {TEXT.lock().unwrap()};
  let index = vec.iter().position(|ref e| ((e.c == '\r') && (e.y == y)));
  index
}
fn getindex() -> Option<usize>
{
  let x = {POS.lock().unwrap().x};
  let y = {POS.lock().unwrap().y};
  let vec = {TEXT.lock().unwrap()};
  let index = vec.iter().position(|ref e| ((e.x == x) && (e.y == y)));
  index 
}
fn getrect(i :usize) -> RECT
{
  let v = {TEXT.lock().unwrap()};
  let w = {*CHX.lock().unwrap()};
  let h = {*CHY.lock().unwrap()};
  let r = RECT{left:v[i].x*w,top:(v[i].y*h),right:(v[i].x*w+w),bottom:(v[i].y*h+h)};
  r
}

fn line(w :HWND, f :HFONT, mode :u8)
{
  let x = {POS.lock().unwrap().x};
  let y = {POS.lock().unwrap().y};
  match mode
  {
    // delete
    0 =>
    {
      if (x == 0 && y == 0) {return;}
      let mut line_end :usize = 0;
      let lastx = getlastx();
      match lastx
      {
        None => {},
        _ => {line_end = lastx.unwrap();},
      }

      if( x > 0 )
      {
        POS.lock().unwrap().x -= 1;
      }
      else if( y > 0 )
      {
        POS.lock().unwrap().y -= 1;
        POS.lock().unwrap().x = getx(line_end);
      }

      let index = getindex();
      match index
      {
        None => {},
        _ =>
        {
          hidecaret(w);
          let index = index.unwrap();
          for i in index..(line_end+1)
          {
            clearch(w, i, 0);
            setx(i, getx(i)-1);
          } 
          remove(index);
          let end = line_end-1;
          for i in index..end
          {
            let ch = CH{x:getx(i),y:gety(i),c:getc(i)};
            drawtext(w, f, ch, 1);
          }
          // workaround, repaint the trailing char with space
          let ch = CH{x:getx(end),y:gety(end),c:' '};
          drawtext(w, f, ch, 1);
          showcaret(w);
        },
      }
    },
    // enter
    1 =>
    {
      let ch = CH{x:x,y:y,c:'\r'};
      save(ch);
    },
    _ => {},
  }
}

fn drawtext(w :HWND, f :HFONT, c :CH, p :WPARAM)
{
  unsafe
  {
    let dc = user::GetDC(w) as HDC;
    gdi::SelectObject(dc, f as HGDIOBJ );
    gdi::SetTextColor(dc, gdi::RGB(R_A,G_A,B_A));
    gdi::SetBkColor(dc, gdi::RGB(R_B,G_B,B_B));

    let string :String = c.c.to_string();
    let ch = to_wstring(&string);
    let ch_w = {*CHX.lock().unwrap()};
    let ch_h = {*CHY.lock().unwrap()};
    gdi::TextOutW(dc, c.x * ch_w, c.y * ch_h, ch.as_ptr(), 1);

    match p {0 => {save(c);}, _ => {},}
    user::ReleaseDC(w, dc);
  }
}

fn key_up(w :HWND)
{
  let y = {POS.lock().unwrap().y};
  if y == 0 {return;}
  hidecaret(w);
  POS.lock().unwrap().y -= 1;
  showcaret(w);
}

fn key_down(w :HWND)
{
  hidecaret(w);
  POS.lock().unwrap().y += 1;
  showcaret(w);
}

fn key_left(w :HWND)
{
  let x = {POS.lock().unwrap().x} as usize;
  if x == 0 {return;}

  *END.lock().unwrap() = 0;
  POS.lock().unwrap().x -= 1;
  let index = getindex();
  match index
  {
    None => {},
    _ =>
    {
      hidecaret(w);
      POS.lock().unwrap().x = getx(index.unwrap());
      showcaret(w);
    },
  }
}

fn key_right(w :HWND)
{
  let index = getindex();
  match index
  {
    None => {},
    _ =>
    {
      if '\r' == getc(index.unwrap()) {return;}
      let length = getlength();
      let end = {*END.lock().unwrap()};
      if (length - 1) == index.unwrap()
      {
        if end != 1
        {
          *END.lock().unwrap() = 1;
          hidecaret(w);
          POS.lock().unwrap().x = getx(index.unwrap()) + 1;
          showcaret(w);
        }
        return;
      }

      hidecaret(w);
      POS.lock().unwrap().x = getx(index.unwrap()+1);
      showcaret(w);
    },
  }
}

fn showcaret(w :HWND)
{
  let index = getindex();
  match index
  {
    None => 
    {
      let i = getlength() + 1;
      match i
      {
        1 => 
        {
          unsafe
          {
            user::SetCaretPos(0,0);
            user::ShowCaret(w);
          }
        },
        _ =>
        {
          let x = {POS.lock().unwrap().x};
          let y = {POS.lock().unwrap().y};
          let ch_w = {*CHX.lock().unwrap()};
          let ch_h = {*CHY.lock().unwrap()};
          unsafe
          {
            user::SetCaretPos(x*ch_w, y*ch_h);
            user::ShowCaret(w);
          }
        },
      }
    },
    _ =>
    {
      let i = index.unwrap();
      let x = getx(i);
      let y = gety(i);
      let c = getc(i);
      let ch_w = {*CHX.lock().unwrap()};
      let ch_h = {*CHY.lock().unwrap()};
      unsafe
      {
        user::SetCaretPos(x*ch_w, y*ch_h);
        user::ShowCaret(w);
      }
    },
  }
}

fn hidecaret(w :HWND)
{
  unsafe{user::HideCaret(w);}
}

fn edit(w :HWND, p :WPARAM, f :HFONT)
{
  if unsafe{user::GetAsyncKeyState(user::VK_CONTROL)} as u16 & 0x8000 != 0
  {
    // println!("p: 0x{0:02x}", p as u8);
    match p 
    {
      // ctrl + o 
      0x0F => 
      {
        let length = getlength();
        println!("debug, length: {0}", length);
        let vec = {TEXT.lock().unwrap()};
        for val in vec.iter() {print!("[{0:02x},{1}]", val.c as u8, val.x);} println!{""};
      },
      // ctrl + x 
      0x18 => 
      {
        let path = String::from("./test.txt");
        fileio(w, f, path, 0); 
      },
      0x13 => println!("0x13"),
      0x02 => println!("0x02"),
      0x03 => println!("0x03"),
      _ => (),
    }
    // println!("GetAsyncKeyState");
    return;
  }

  let mode = {*MODE.lock().unwrap()};
  match mode
  {
    // save mode
    0 =>
    unsafe
    {
      hidecaret(w);
      match p
      {
        // 0: move key to x 0
        0x30 => 
        {
          *END.lock().unwrap() = 0;
          let y = {POS.lock().unwrap().y};
          let ch_h = {*CHY.lock().unwrap()};
          user::SetCaretPos(0, y*ch_h);
          POS.lock().unwrap().x = 0;
        },
        // $: move key to x end
        0x24 => 
        {
          let index = getindex();
          match index
          {
            None => {},
            _ =>
            {
              let index = index.unwrap();
              let length = getlength();
              let y = {POS.lock().unwrap().y};
              let w = {*CHX.lock().unwrap()} as i32;
              let h = {*CHY.lock().unwrap()};
              for i in index..length
              {
                if getc(i) == '\r' 
                {
                  let x = getx(i);
                  user::SetCaretPos(x*w, y*h);
                  POS.lock().unwrap().x = x;
                  break;
                }
                else if i == (length-1)
                {
                  *END.lock().unwrap() = 1;
                  let x = getx(i)+1;
                  user::SetCaretPos(x*w, y*h);
                  POS.lock().unwrap().x = x;
                }
              }
            },
          }
        },
        // a: change mode 
        0x61 => 
        {
          *MODE.lock().unwrap() = 1;
        },
        // i: change mode 
        0x69 => 
        {
          *MODE.lock().unwrap() = 2;
        },
        // h, j, k, l
        0x68 => key_left(w), 0x6A => key_down(w), 0x6B => key_up(w), 0x6C => key_right(w),
        // key dd
        0x64 => println!("0x64"),
        // key zz
        0x7A => println!("0x7A"),
        _ => (),
      }
      user::ShowCaret(w);
      return;
    },
    // edit mode, bypass
    _ => (),
  }

  match p 
  {
    // backspace
    0x08 => 
    {
      line(w, f, 0);
    },
    // enter 
    0x0D => 
    {
      line(w, f, 1);
      hidecaret(w);
      POS.lock().unwrap().x = 0;
      POS.lock().unwrap().y += 1;
      showcaret(w);
    },
    // esc
    0x1B => 
    {
      *MODE.lock().unwrap() = 0;
    },
    _ => 
    // edit
    unsafe
    {
      let x = {POS.lock().unwrap().x};
      let y = {POS.lock().unwrap().y};
      let index = getindex();
      match index
      {
        None => 
        {
          hidecaret(w);
          let c = std::char::from_u32_unchecked(p as u32);
          let ch = CH{x:x as i32,y:y as i32,c:c};
          drawtext(w, f, ch, 0);  
          POS.lock().unwrap().x += 1;
          showcaret(w);
        },
        _ => 
        {
          let index = index.unwrap();
          let length = getlength();
          if index < length
          {
            let mut line_end  = length+1;
            hidecaret(w);
            for i in index..length
            {
              let c = getc(i);
              if '\r' == c 
              {
                line_end = i+1;
                setx(i, getx(i)+1);
                break;
              }
              else
              {
                clearch(w, i, 0);
                setx(i, getx(i)+1);
              }
            }
            let c = std::char::from_u32_unchecked(p as u32);
            let ch = CH{x:x as i32,y:y as i32,c:c};
            insert(index, ch);

            for i in index..line_end
            {
              let ch = CH{x:getx(i) as i32,y:gety(i) as i32,c:getc(i)};
              drawtext(w, f, ch, 1);  
            }
            POS.lock().unwrap().x += 1;
            showcaret(w);
          }
          else
          {
            println!("index >= length");
          }
        },
      }
    },
  }
}

pub unsafe extern "system" fn window_proc(w :HWND, 
  msg :UINT, p :WPARAM, l :LPARAM) -> LRESULT
{
  if msg == user::WM_CREATE
  {
    let param = &*(l as LPCREATESTRUCTW);
    // user::SetWindowLongPtrW(w, user::GWLP_USERDATA,  param.lpCreateParams as isize); // 64bit
    user::SetWindowLongPtrW(w, user::GWLP_USERDATA,  param.lpCreateParams as i32);      // 32bit
  }

  match msg 
  {
    user::WM_CHAR => 
    {
      let font = user::GetWindowLongPtrW(w, user::GWLP_USERDATA) as HFONT;
      edit(w, p, font)
    },
    user::WM_DROPFILES => 
    {
      let hdrop = p as shell::HDROP;
      let file = shell::DragQueryFileW(hdrop, 0xFFFFFFFF, ptr::null_mut(), 0);

      if file != 1
      {
        println!("multiple files not supported");
        shell::DragFinish(hdrop);
      }
      else
      {
        let mut v = vec![0u16; MAX_PATH as usize];
        shell::DragQueryFileW(hdrop, 0, v.as_mut_ptr(), MAX_PATH as u32);
        shell::DragFinish(hdrop);

        let mut path = String::new();
        for val in v.iter() 
        {
          let c = (*val & 0xFF) as u8;
          if c == 0 {break;} else { path.push(c as char);}
        } 
        // println!("file path : {:?}", path);
        let font = user::GetWindowLongPtrW(w, user::GWLP_USERDATA) as HFONT;
        fileio(w, font, path, 0); 
      }
    },
    user::WM_DESTROY => 
    {
      shell::DragAcceptFiles(w, 0);
      user::PostQuitMessage(0);
    },
    _ => (),
  }
  return user::DefWindowProcW( w, msg, p, l);
}

// main loop
////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn main() 
{
  unsafe
  {
    let class_name = to_wstring("window");
    let font_name = to_wstring("Dejavu Sans Mono");
    let icon_name = to_wstring("./read.ico");

    let font = gdi::CreateFontW(18, 0, 0, 0, 
      gdi::FW_LIGHT, 0, 0, 0, gdi::DEFAULT_CHARSET, gdi::OUT_DEFAULT_PRECIS, 
      gdi::CLIP_DEFAULT_PRECIS, gdi::DEFAULT_QUALITY,  gdi::DEFAULT_PITCH, 
      font_name.as_ptr());

    let wnd = WNDCLASSW
    {
      style: 0,
      lpfnWndProc: Some(window_proc),
      cbClsExtra: 0,
      cbWndExtra: 0,
      hInstance: 0 as HINSTANCE,
      // hIcon: user::LoadIconW(0 as HINSTANCE, user::IDI_APPLICATION),
      hIcon: user::LoadImageW(0 as HINSTANCE, icon_name.as_ptr(), user::IMAGE_ICON, 0, 0, 
        user::LR_LOADFROMFILE | user::LR_DEFAULTSIZE | user::LR_SHARED) as HICON,
      hCursor: 0 as HICON, // user32::LoadCursorW(0 as HINSTANCE, user::IDI_APPLICATION),
      hbrBackground: 0 as HBRUSH,
      lpszMenuName: 0 as LPCWSTR,
      lpszClassName: class_name.as_ptr(),
    };

    user::RegisterClassW(&wnd);

    let hwnd = user::CreateWindowExW(0, class_name.as_ptr(), 
      to_wstring("read v0.1").as_ptr(),
      WS_OVERLAPPEDWINDOW | WS_VISIBLE, 0, 0, W, H, 0 as HWND, 0 as HMENU, 0 as HINSTANCE, font as LPVOID);
  
    user::ShowWindow(hwnd, user::SW_SHOW);
      
    let mut msg = user::MSG 
    {
      hwnd : 0 as HWND,
      message : 0 as UINT, 
      wParam : 0 as WPARAM,
      lParam : 0 as LPARAM,
      time : 0 as DWORD,
      pt : POINT{x:0, y:0, }, 
    };

    // font
    if font != ptr::null_mut()
    {
      user::SendMessageW(hwnd, user::WM_SETFONT, font as WPARAM, 1);
    }

    let dc = user::GetDC(hwnd) as HDC;
    let mut char_w : INT = 0;
    let mut tm = gdi::TEXTMETRICW
    {
      tmHeight: 0,
      tmAscent: 0,
      tmDescent: 0,
      tmInternalLeading: 0,
      tmExternalLeading: 0,
      tmAveCharWidth: 0,
      tmMaxCharWidth: 0,
      tmWeight: 0,
      tmOverhang: 0,
      tmDigitizedAspectX: 0,
      tmDigitizedAspectY: 0,
      tmFirstChar: 0,
      tmLastChar: 0,
      tmDefaultChar: 0,
      tmBreakChar: 0,
      tmItalic: 0,
      tmUnderlined: 0,
      tmStruckOut: 0,
      tmPitchAndFamily: 0,
      tmCharSet: 0,
    };
    gdi::SelectObject(dc, font as HGDIOBJ );
    gdi::GetCharWidth32W(dc, 0 as UINT, 0 as UINT, &mut char_w); 
    gdi::GetTextMetricsW(dc, &mut tm);
    user::ReleaseDC(hwnd, dc);
    *CHX.lock().unwrap() = char_w;
    *CHY.lock().unwrap() = tm.tmHeight;
    user::CreateCaret(hwnd, 0 as HBITMAP, 1, tm.tmHeight);
    showcaret(hwnd);
    shell::DragAcceptFiles(hwnd, 1);
    // background
    // let brush = gdi::CreateSolidBrush(gdi::RGB(R_B,G_B,B_B)) as isize; // 64bit
    let brush = gdi::CreateSolidBrush(gdi::RGB(R_B,G_B,B_B)) as i32;      // 32bit
    user::SetClassLongPtrW(hwnd, user::GCLP_HBRBACKGROUND, brush);
    user::MoveWindow(hwnd, SX, SY, W, H, 1);
    loop
    {
      let m = user::GetMessageW(&mut msg, 0 as HWND, 0, 0);
      if msg.message == user::WM_QUIT {break;}
      if m > 0
      {
        user::TranslateMessage(&mut msg);
        user::DispatchMessageW(&mut msg);
      }
    }
  }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////
