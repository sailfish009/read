// hide console window
// #![windows_subsystem = "windows"]

const SX:i32 = 200; // window x
const SY:i32 = 200; // window y
const W:i32 = 800;  // window width
const H:i32 = 600;  // window height

extern crate winapi; 
use winapi::um::winuser as user;
use winapi::um::wingdi as gdi;
use winapi::shared::windef as def;
use winapi::shared::minwindef as mindef;

use def::HWND;
use def::HMENU;
use def::HBRUSH;
use def::HICON;

use mindef::HINSTANCE;
use mindef::UINT;
use mindef::DWORD;
use mindef::WPARAM;
use mindef::LPARAM;
use mindef::LRESULT;

use winapi::um::winnt::LPCWSTR;
use user::WS_OVERLAPPEDWINDOW;
use user::WS_VISIBLE;
use user::WNDCLASSW;

use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;
use std::ptr;

fn to_wstring(str : &str) -> Vec<u16> 
{
  let v : Vec<u16> =
    OsStr::new(str).encode_wide().chain(Some(0).into_iter()).collect();
  v
}

pub unsafe extern "system" fn window_proc(h_wnd :HWND, 
  msg :UINT, w_param :WPARAM, l_param :LPARAM) -> LRESULT
{
  if msg == user::WM_DESTROY 
  {
    user::PostQuitMessage(0);
  }
  return user::DefWindowProcW( h_wnd, msg, w_param, l_param);
}

fn main() 
{
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
      hbrBackground: 0 as HBRUSH,
      lpszMenuName: 0 as LPCWSTR,
      lpszClassName: class_name.as_ptr(),
    };

    user::RegisterClassW(&wnd);

    let hwnd = user::CreateWindowExW(0, class_name.as_ptr(), 
      to_wstring("read v0.1").as_ptr(),
      WS_OVERLAPPEDWINDOW | WS_VISIBLE, 0, 0, W, H, 0 as HWND, 0 as HMENU, 0 as HINSTANCE, ptr::null_mut());
  
    user::InvalidateRect(hwnd, ptr::null(), 1);
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
    let brush  =  gdi::CreateSolidBrush(gdi::RGB(0,0,0)) as i32;
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
