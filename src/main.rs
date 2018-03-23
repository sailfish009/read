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
#![windows_subsystem = "windows"]

// #[macro_use]
// extern crate lazy_static;

const SX:i32 = 200;  const SY:i32 = 200;  const W:i32 = 800;   const H:i32 = 600;
const R_A: u8 = 0;   const G_A: u8 = 0;   const B_A: u8 = 0;
const R_B: u8 = 250; const G_B: u8 = 250; const B_B: u8 = 250;

extern crate winapi; 

use winapi::um::winuser as user;     use winapi::um::wingdi as gdi;
use winapi::shared::windef as def;   use winapi::shared::minwindef as mindef;
use def::HWND;                       use def::HMENU;        
use def::HBRUSH;                     use def::HICON;
use def::HFONT;                      use def::HGDIOBJ;
use def::HDC;     
use mindef::HINSTANCE;               use mindef::UINT;
use mindef::DWORD;                   use mindef::WPARAM;
use mindef::LPARAM;                  use mindef::LRESULT;
use user::WS_OVERLAPPEDWINDOW;       use user::WS_VISIBLE;use user::WNDCLASSW;
use winapi::um::winnt::LPCWSTR;      use winapi::um::winnt::LONG;

use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;

use std::ptr;
// use std::string::String;

static MODE: u8 = 0;
static LINE: &str = "";
static CH_Y: LONG = 0;

struct CH 
{
  x: LONG,
  y: LONG,
  c: u8,
  w: u8,
}

static CHAR: CH = CH {x:0,y:0,c:0,w:0};

fn to_wchar(str : &str) -> *const u16 {
  let v : Vec<u16> =
    OsStr::new(str).encode_wide(). chain(Some(0).into_iter()).collect();
  v.as_ptr()
}

fn to_wstring(str : &str) -> Vec<u16> 
{
  let v : Vec<u16> =
    OsStr::new(str).encode_wide().chain(Some(0).into_iter()).collect();
  v
}

fn drawtext(w :HWND, f: HFONT, c :CH, p :WPARAM, l: LPARAM)
{
  unsafe
  {
    let dc = user::GetDC(w) as HDC;
    gdi::SelectObject(dc, f as HGDIOBJ );
    gdi::SetTextColor(dc, gdi::RGB(R_B,G_B,B_B));
    gdi::SetBkColor(dc, gdi::RGB(R_A,G_A,B_A));

    match p 
    {
      0 =>
      {
        let ch = c.c as LPCWSTR;

        if l == 0
        {
          gdi::TextOutW(dc, c.x, c.y * CH_Y, ch, 1);
        }
      },
      _ => (),
    }
  }
}



fn edit(w :HWND, p :WPARAM)
{
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
    	// LINE.push_str()
    }
    ,
  }
}

pub unsafe extern "system" fn window_proc(w :HWND, 
  msg :UINT, p :WPARAM, l :LPARAM) -> LRESULT
{
  match msg 
  {
    user::WM_CREATE => println!("init"),
    user::WM_CHAR => edit(w, p),
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

    let font = gdi::CreateFontW(18, 0, 0, 0, 
      gdi::FW_LIGHT, 0, 0, 0, gdi::DEFAULT_CHARSET, gdi::OUT_DEFAULT_PRECIS, 
      gdi::CLIP_DEFAULT_PRECIS, gdi::DEFAULT_QUALITY,  gdi::DEFAULT_PITCH, 
      to_wchar("Dejavu Sans Mono") );

    let wnd = WNDCLASSW
    {
      style: 0,
      lpfnWndProc: Some(window_proc),
      cbClsExtra: 0,
      cbWndExtra: 0,
      hInstance: 0 as HINSTANCE,
      hIcon: 0 as HICON, // user32::LoadIconW(0 as HINSTANCE, winapi::um::winuser::IDI_APPLICATION),
      hCursor: 0 as HICON, // user32::LoadCursorW(0 as HINSTANCE, winapi::um::winuser::IDI_APPLICATION),
      hbrBackground: 0 as HBRUSH,
      lpszMenuName: 0 as LPCWSTR,
      lpszClassName: class_name.as_ptr(),
    };

    user::RegisterClassW(&wnd);

    let hwnd = user::CreateWindowExW(0, class_name.as_ptr(), 
      to_wstring("read v0.1").as_ptr(),
      WS_OVERLAPPEDWINDOW | WS_VISIBLE, 0, 0, W, H, 0 as HWND, 0 as HMENU, 0 as HINSTANCE, ptr::null_mut());
  
    // user::InvalidateRect(hwnd, ptr::null(), 1);
    user::ShowWindow(hwnd, user::SW_SHOW);
      
    let mut msg = user::MSG 
    {
      hwnd : 0 as HWND,
      message : 0 as UINT, 
      wParam : 0 as WPARAM,
      lParam : 0 as LPARAM,
      time : 0 as DWORD,
      pt : winapi::shared::windef::POINT{x:0, y:0, }, 
    };

    // background
    let brush = gdi::CreateSolidBrush(gdi::RGB(R_A,G_A,B_A)) as i32;
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
