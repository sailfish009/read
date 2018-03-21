extern crate winapi; 
use winapi::shared::windef::HWND;
// use winapi::windef::HWND;
use winapi::shared::windef::HMENU;
use winapi::shared::windef::HBRUSH;
use winapi::shared::windef::HICON;
use winapi::shared::minwindef::HINSTANCE;
// use winapi::minwindef::HINSTANCE;

use winapi::shared::minwindef::UINT;
use winapi::shared::minwindef::DWORD;
use winapi::shared::minwindef::WPARAM;
use winapi::shared::minwindef::LPARAM;
use winapi::shared::minwindef::LRESULT;
use winapi::um::winnt::LPCWSTR;

use winapi::um::winuser::WS_OVERLAPPEDWINDOW;
use winapi::um::winuser::WS_VISIBLE;
use winapi::um::winuser::WNDCLASSW;

extern crate user32; 
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;

fn to_wstring(str : &str) -> Vec<u16> 
{
    let v : Vec<u16> =
            OsStr::new(str).encode_wide().chain(Some(0).into_iter()).collect();
    v
}

pub unsafe extern "system" fn window_proc(h_wnd :HWND, 
	msg :UINT, w_param :WPARAM, l_param :LPARAM) -> LRESULT
{
    if msg == winapi::um::winuser::WM_DESTROY {
        user32::PostQuitMessage(0);
    }

    return user32::DefWindowProcW( h_wnd, msg, w_param, l_param);
}

fn main() 
{
  // println!("Hello, world!");
  unsafe
  {
    let class_name = to_wstring("window");

    let wnd = WNDCLASSW
    {
      style: 0,
	  lpfnWndProc: Some(window_proc),
      cbClsExtra: 0,
      cbWndExtra: 0,
      hInstance: 0 as HINSTANCE,
      hIcon: 0 as HICON, // user32::LoadIconW(0 as HINSTANCE, winapi::um::winuser::IDI_APPLICATION),
      hCursor: 0 as HICON, // user32::LoadCursorW(0 as HINSTANCE, winapi::um::winuser::IDI_APPLICATION),
      hbrBackground: 16 as HBRUSH,
      lpszMenuName: 0 as LPCWSTR,
      lpszClassName: class_name.as_ptr(),
    };

    user32::RegisterClassW(&wnd);

    let hwnd = winapi::um::winuser::CreateWindowExW(0, class_name.as_ptr(), 
      to_wstring("read v0.1").as_ptr(),
      WS_OVERLAPPEDWINDOW | WS_VISIBLE, 0, 0, 640, 480, 0 as HWND, 0 as HMENU, 0 as HINSTANCE, std::ptr::null_mut());
  
    user32::ShowWindow(hwnd, winapi::um::winuser::SW_SHOW);
      
    let mut msg = winapi::um::winuser::MSG 
    {
      hwnd : 0 as HWND,
      message : 0 as UINT, 
      wParam : 0 as WPARAM,
      lParam : 0 as LPARAM,
      time : 0 as DWORD,
      p : winapi::shared::windef::POINT{x:0, y:0, }, 
    };

    loop
    {
      let m = user32::GetMessageW(&mut msg, 0 as HWND, 0, 0);
      if m > 0
      {
        user32::TranslateMessage(&mut msg);
        user32::DispatchMessageW(&mut msg);
      }
    }
  }
}
