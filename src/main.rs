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

const SX: i32 = 200; const SY: i32 = 200; const W:  i32 = 800;   const H:i32 = 600;
const R_A: u8 = 250; const G_A: u8 = 250; const B_A: u8 = 250;
const R_B: u8 = 0;   const G_B: u8 = 0;   const B_B: u8 = 0;

extern crate winapi; 

use winapi::um::winuser as user;     
use winapi::um::wingdi as gdi;
use winapi::shared::windef::{HWND, HMENU, HBRUSH, HICON, HFONT, HGDIOBJ, HBITMAP, HDC, POINT};        
use winapi::shared::minwindef::{HINSTANCE, INT, UINT, LPINT, DWORD, WPARAM, LPARAM, LRESULT, LPVOID };        
use user::{WS_OVERLAPPEDWINDOW, WS_VISIBLE, WNDCLASSW, LPCREATESTRUCTW};        
use winapi::um::winnt::{LPCWSTR, LONG};
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;
use std::ptr;
use std::string::String;
use std::sync::Mutex;
use std::iter::Zip;

static MODE: u8 = 0;
static CH_Y: LONG = 0;
struct CH {x: LONG,  y: LONG, c: char, w: INT}

// static CHAR: CH = CH {x:0,y:0,c:0,w:0};

lazy_static!
{
  static ref LA: Mutex<Vec<LINE>> = Mutex::new(Vec::new());
  static ref LINE: Mutex<Vec<CH>> = Mutex::new(Vec::new());
  static ref POS: Mutex<POINT> = Mutex::new(POINT{x:0, y:0});
  static ref CHX: Mutex<LONG> = Mutex::new(0);
  static ref CHY: Mutex<LONG> = Mutex::new(0);
}

fn to_wchar(str : &str) -> *const u16 
{
  let v : Vec<u16> =
    OsStr::new(str).encode_wide().chain(Some(0).into_iter()).collect();
  v.as_ptr()
}

fn to_wstring(str : &str) -> Vec<u16> 
{
  let v : Vec<u16> =
    OsStr::new(str).encode_wide().chain(Some(0).into_iter()).collect();
  v
}

fn drawtext(w :HWND, f :HFONT, mut c :CH, p :WPARAM, l :LPARAM)
{
  unsafe
  {
    let dc = user::GetDC(w) as HDC;
    gdi::SelectObject(dc, f as HGDIOBJ );
    gdi::SetTextColor(dc, gdi::RGB(R_A,G_A,B_A));
    gdi::SetBkColor(dc, gdi::RGB(R_B,G_B,B_B));

    match p 
    {
      0 =>
      {
        if l == 0
        {
          let string :String = c.c.to_string();
          let ch = to_wstring(&string);
          let mut char_w : INT = 0;
          c.x = POS.lock().unwrap().x;
          gdi::GetCharWidth32W(dc, 0 as UINT, 0 as UINT, &mut char_w); 
          gdi::TextOutW(dc, c.x, c.y * CH_Y, ch.as_ptr(), 1);
          POS.lock().unwrap().x += char_w;
        }
      },
      _ => (),
    }
    user::ReleaseDC(w, dc);
  }

  // LINE.lock().unwrap().push(c);
}


fn edit(w :HWND, p :WPARAM, f :HFONT)
{
  println!("font: {:?}", f);
  if unsafe{user::GetAsyncKeyState(user::VK_CONTROL)} as u16 & 0x8000 != 0
  {
    unsafe{user::HideCaret(w)};
    match p 
    {
      0x0F => println!("0x0F"),
      0x13 => println!("0x13"),
      0x02 => println!("0x02"),
      0x03 => println!("0x03"),
      _ => (),
    }
    println!("GetAsyncKeyState");
    return;
  }

  match MODE 
  {
    // save mode
    1 =>
    unsafe
    {
      user::HideCaret(w);
      match p
      {
        // key move
        0x69 => println!("0x69"),
        0x68 => println!("0x68"),
        0x6C => println!("0x6C"),
        0x6B => println!("0x6B"),
        0x6A => println!("0x6A"),
        // key dd
        0x64 => println!("0x64"),
        // key zz
        0x7A => println!("0x7A"),
        _ => (),
      }
      user::ShowCaret(w);
    }
    ,
    // edit mode, bypass
    _ => (),
  }

  match p 
  {
    // backspace
    0x08 => println!("0x08"),
    // enter 
    0x0D => println!("0x0D"),
    // esc
    0x1B => println!("0x1B"),
    _ => 
    // edit
    unsafe
    {
      let d = std::char::from_u32_unchecked(p as u32);
      let ch = CH{x:0,y:0,c:d,w:0};
      drawtext(w, f, ch, 0, 0);  
    }
    ,
  }
}

pub unsafe extern "system" fn window_proc(w :HWND, 
  msg :UINT, p :WPARAM, l :LPARAM) -> LRESULT
{
  if msg == user::WM_CREATE
  {
    let param = &*(l as LPCREATESTRUCTW);
    user::SetWindowLongPtrW(w, user::GWLP_USERDATA,  param.lpCreateParams as i32);
  }

  let font = user::GetWindowLongPtrW(w, user::GWLP_USERDATA) as HFONT;
  match msg 
  {
    user::WM_CHAR => edit(w, p, font),
    user::WM_DESTROY => user::PostQuitMessage(0),
    _ => (),
  }
  return user::DefWindowProcW( w, msg, p, l);
}

fn main() 
{
  unsafe
  {
    let class_name = to_wstring("window");
    let font_name = to_wstring("Dejavu Sans Mono");

    let font = gdi::CreateFontW(18, 0, 0, 0, 
      gdi::FW_LIGHT, 0, 0, 0, gdi::DEFAULT_CHARSET, gdi::OUT_DEFAULT_PRECIS, 
      gdi::CLIP_DEFAULT_PRECIS, gdi::DEFAULT_QUALITY,  gdi::DEFAULT_PITCH, 
      font_name.as_ptr());

    println!("font: {:?}", font);

    let wnd = WNDCLASSW
    {
      style: 0,
      lpfnWndProc: Some(window_proc),
      cbClsExtra: 0,
      cbWndExtra: 0,
      hInstance: 0 as HINSTANCE,
      hIcon: 0 as HICON, // user32::LoadIconW(0 as HINSTANCE, user::IDI_APPLICATION),
      hCursor: 0 as HICON, // user32::LoadCursorW(0 as HINSTANCE, user::IDI_APPLICATION),
      hbrBackground: 0 as HBRUSH,
      lpszMenuName: 0 as LPCWSTR,
      lpszClassName: class_name.as_ptr(),
    };

    user::RegisterClassW(&wnd);

    let hwnd = user::CreateWindowExW(0, class_name.as_ptr(), 
      to_wstring("read v0.1").as_ptr(),
      WS_OVERLAPPEDWINDOW | WS_VISIBLE, 0, 0, W, H, 0 as HWND, 0 as HMENU, 0 as HINSTANCE, font as LPVOID);
    // WS_OVERLAPPEDWINDOW | WS_VISIBLE, 0, 0, W, H, 0 as HWND, 0 as HMENU, 0 as HINSTANCE, ptr::null_mut());
  
    // user::InvalidateRect(hwnd, ptr::null(), 1);
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
    user::SendMessageW(hwnd, user::WM_SETFONT, font as WPARAM, 1);

    let dc = user::GetDC(hwnd) as HDC;
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
    gdi::GetTextMetricsW(dc, &mut tm);
    user::ReleaseDC(hwnd, dc);

    *CHX.lock().unwrap() = tm.tmAveCharWidth;
    *CHY.lock().unwrap() = tm.tmHeight;

    // println!("char_x: {0}, char_y: {1}", *CHX.lock().unwrap(), *CHY.lock().unwrap());
    user::CreateCaret(hwnd, 0 as HBITMAP, 1, *CHY.lock().unwrap());

    // background
    let brush = gdi::CreateSolidBrush(gdi::RGB(R_B,G_B,B_B)) as i32;
    user::SetClassLongPtrW(hwnd, user::GCLP_HBRBACKGROUND, brush);
    user::MoveWindow(hwnd, SX, SY, W, H, 1);

    loop
    {
      let m = user::GetMessageW(&mut msg, 0 as HWND, 0, 0);
      if msg.message == user::WM_QUIT
      {
        break;
      }
      if m > 0
      {
        user::TranslateMessage(&mut msg);
        user::DispatchMessageW(&mut msg);
      }
    }
  }
}
