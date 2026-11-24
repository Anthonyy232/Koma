//! Drives Koma on the real desktop and looks at the result: a development tool.
//!
//!   peek run:target/debug/Koma.exe wait:1500 drag:300,300,900,700 shot:out.png key:escape kill
//!
//! Steps run in order. Input goes only to Koma: keys while its window has the keyboard, the
//! pointer only onto its window, at physical pixels from that window's top left. A `shot` crop
//! is in the desktop's pixels. `global` is the exception: a chord for Koma's global shortcut,
//! sent to whatever has the keyboard.

use std::{fs::File, io::BufWriter, process::Child, thread::sleep, time::Duration};
use windows_sys::Win32::{
    Foundation::{CloseHandle, HWND, LPARAM, POINT, RECT},
    Graphics::Gdi::{
        BitBlt, ClientToScreen, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject,
        GetDC, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, CAPTUREBLT, DIB_RGB_COLORS,
        SRCCOPY,
    },
    System::DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard},
    System::Memory::{GlobalLock, GlobalSize, GlobalUnlock},
    System::Ole::CF_DIB,
    System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
    },
    UI::{
        HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
        Input::KeyboardAndMouse::*,
        WindowsAndMessaging::{
            EnumWindows, GetForegroundWindow, GetSystemMetrics, GetWindowRect,
            GetWindowThreadProcessId, IsWindowVisible, WindowFromPoint, SM_CXVIRTUALSCREEN,
            SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
        },
    },
};

fn main() {
    unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
    let mut child: Option<Child> = None;
    for step in std::env::args().skip(1) {
        let (verb, rest) = step.split_once(':').unwrap_or((&step, ""));
        match verb {
            "run" => {
                let mut parts = rest.split(',');
                let exe = parts.next().unwrap();
                let mut launch = std::process::Command::new(exe)
                    .args(parts)
                    .spawn()
                    .expect("run");
                // Koma is one process: a launch while it runs hands over to it and ends.
                if child.is_some() {
                    launch.wait().expect("run");
                } else {
                    child = Some(launch);
                }
            }
            "wait" => sleep(Duration::from_millis(rest.parse().expect("wait:<ms>"))),
            "shot" => {
                let (path, rect) = match rest.split_once('@') {
                    Some((path, crop)) => {
                        let n = numbers(crop);
                        (
                            path,
                            Rect {
                                x: n[0],
                                y: n[1],
                                width: n[2],
                                height: n[3],
                            },
                        )
                    }
                    None => (rest, desktop()),
                };
                save(
                    path,
                    rect.width as usize,
                    rect.height as usize,
                    capture(rect),
                );
            }
            "clip" => clipboard_image(rest),
            "exit" => {
                let code = child.as_mut().unwrap().wait().expect("exit").code();
                println!("exit: {code:?}");
            }
            "kill" => stop(child.as_mut().unwrap()),
            "global" => chord(rest),
            "key" | "text" => {
                if !is_koma(unsafe { GetForegroundWindow() }) {
                    refuse(child.as_mut(), "Koma does not have the keyboard");
                }
                if verb == "key" {
                    chord(rest);
                } else {
                    rest.encode_utf16().for_each(|unit| {
                        key(0, unit, KEYEVENTF_UNICODE);
                        key(0, unit, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP);
                    });
                }
            }
            "move" | "click" | "drag" => {
                let n = numbers(rest);
                // Per point: a display waking mid-drag moves the window.
                let on_screen = |x: i32, y: i32| {
                    let mut point = POINT { x, y };
                    unsafe { ClientToScreen(koma(), &mut point) };
                    point
                };
                if !is_koma(unsafe { WindowFromPoint(on_screen(n[0], n[1])) }) {
                    refuse(child.as_mut(), "that point is not on Koma's window");
                }
                let at = |x: i32, y: i32, flags: u32| {
                    let point = on_screen(x, y);
                    pointer(point.x, point.y, flags)
                };
                at(n[0], n[1], 0);
                if verb != "move" {
                    at(n[0], n[1], MOUSEEVENTF_LEFTDOWN);
                }
                if verb == "drag" {
                    for i in 1..=12 {
                        sleep(Duration::from_millis(20));
                        at(
                            n[0] + (n[2] - n[0]) * i / 12,
                            n[1] + (n[3] - n[1]) * i / 12,
                            0,
                        );
                    }
                }
                if verb != "move" {
                    sleep(Duration::from_millis(40));
                    let end = if verb == "drag" { 2 } else { 0 };
                    at(n[end], n[end + 1], MOUSEEVENTF_LEFTUP);
                }
            }
            _ => panic!("unknown step `{step}`"),
        }
    }
}

fn stop(child: &mut Child) {
    let _ = std::process::Command::new("taskkill")
        .args(["/F", "/T", "/PID", &child.id().to_string()])
        .output();
    let _ = child.wait();
}

fn numbers(text: &str) -> Vec<i32> {
    text.split(',')
        .map(|n| n.trim().parse().expect("a number"))
        .collect()
}

/// Stop Koma and give up, rather than send input anywhere else.
fn refuse(child: Option<&mut Child>, why: &str) -> ! {
    eprintln!("peek: {why}; no input sent");
    if let Some(child) = child {
        stop(child);
    }
    std::process::exit(2)
}

fn is_koma(window: HWND) -> bool {
    let mut owner = 0;
    let mut name = [0u16; 1024];
    let mut length = name.len() as u32;
    unsafe {
        GetWindowThreadProcessId(window, &mut owner);
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, owner);
        QueryFullProcessImageNameW(process, 0, name.as_mut_ptr(), &mut length);
        CloseHandle(process);
    }
    String::from_utf16_lossy(&name[..length as usize]).ends_with("Koma.exe")
}

/// Koma's largest: tooltips and their shadows are Koma's windows too.
fn koma() -> HWND {
    unsafe extern "system" fn visit(window: HWND, largest: LPARAM) -> i32 {
        let (found, area) = &mut *(largest as *mut (HWND, i32));
        let mut rect = RECT::default();
        GetWindowRect(window, &mut rect);
        let size = (rect.right - rect.left) * (rect.bottom - rect.top);
        if size > *area && IsWindowVisible(window) != 0 && is_koma(window) {
            (*found, *area) = (window, size);
        }
        1
    }
    let mut largest: (HWND, i32) = (std::ptr::null_mut(), 0);
    unsafe { EnumWindows(Some(visit), &raw mut largest as LPARAM) };
    largest.0
}

fn pointer(x: i32, y: i32, flags: u32) {
    let Rect {
        x: left,
        y: top,
        width,
        height,
    } = desktop();
    send(INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: ((x - left) * 65535 + width / 2) / (width - 1),
                dy: ((y - top) * 65535 + height / 2) / (height - 1),
                dwFlags: flags | MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
                ..Default::default()
            },
        },
    });
}

fn key(vk: u16, scan: u16, flags: u32) {
    send(INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: scan,
                dwFlags: flags,
                ..Default::default()
            },
        },
    });
}

fn send(input: INPUT) {
    unsafe { SendInput(1, &input, size_of::<INPUT>() as i32) };
    sleep(Duration::from_millis(5));
}

/// `ctrl+shift+z`, `escape`, `enter`, `a`.
fn chord(text: &str) {
    let keys: Vec<u16> = text
        .split('+')
        .map(|name| match name {
            "ctrl" => VK_CONTROL,
            "shift" => VK_SHIFT,
            "escape" => VK_ESCAPE,
            "enter" => VK_RETURN,
            "printscreen" => VK_SNAPSHOT,
            one if one.len() == 1 => one.to_ascii_uppercase().as_bytes()[0] as u16,
            other => panic!("unknown key `{other}`"),
        })
        .collect();
    keys.iter().for_each(|&vk| key(vk, 0, 0));
    keys.iter()
        .rev()
        .for_each(|&vk| key(vk, 0, KEYEVENTF_KEYUP));
}

fn save(path: &str, width: usize, height: usize, bgra: Vec<u8>) {
    let rgb: Vec<u8> = bgra
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|p| [p[2], p[1], p[0]])
        .collect();
    let file = BufWriter::new(File::create(path).expect(path));
    let mut encoder = png::Encoder::new(file, width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgb);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&rgb)
        .unwrap();
    println!("shot: {path} {width}x{height}");
}

/// The clipboard's bitmap (`CF_DIB`), written as a PNG.
fn clipboard_image(path: &str) {
    unsafe {
        OpenClipboard(0 as HWND);
        let handle = GetClipboardData(u32::from(CF_DIB));
        if handle.is_null() {
            CloseClipboard();
            println!("clip: no image on the clipboard");
            return;
        }
        let bytes = std::slice::from_raw_parts(GlobalLock(handle).cast::<u8>(), GlobalSize(handle));
        let header = i32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
        let width = i32::from_le_bytes(bytes[4..8].try_into().unwrap());
        let height = i32::from_le_bytes(bytes[8..12].try_into().unwrap());
        let bits = u16::from_le_bytes(bytes[14..16].try_into().unwrap());
        // `BI_BITFIELDS` puts three colour masks between the header and the pixels.
        let masks = if bytes[16] == 3 { 12 } else { 0 };
        assert_eq!(bits, 32, "clip: expected a 32-bit bitmap");
        let rows = height.unsigned_abs() as usize;
        let stride = width as usize * 4;
        let mut bgra = Vec::with_capacity(stride * rows);
        for row in 0..rows {
            let source = if height > 0 { rows - 1 - row } else { row };
            bgra.extend_from_slice(&bytes[header + masks + source * stride..][..stride]);
        }
        GlobalUnlock(handle);
        CloseClipboard();
        save(path, width as usize, rows, bgra);
    }
}

/// A rectangle of the virtual desktop, in physical pixels.
struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

fn desktop() -> Rect {
    unsafe {
        Rect {
            x: GetSystemMetrics(SM_XVIRTUALSCREEN),
            y: GetSystemMetrics(SM_YVIRTUALSCREEN),
            width: GetSystemMetrics(SM_CXVIRTUALSCREEN),
            height: GetSystemMetrics(SM_CYVIRTUALSCREEN),
        }
    }
}

/// What the screen shows in `rect`: BGRA rows, top first, whose alpha means nothing.
fn capture(rect: Rect) -> Vec<u8> {
    let mut pixels = vec![0u8; rect.width as usize * rect.height as usize * 4];
    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: rect.width,
            // Negative: rows top first.
            biHeight: -rect.height,
            biPlanes: 1,
            biBitCount: 32,
            ..Default::default()
        },
        ..Default::default()
    };
    unsafe {
        let screen = GetDC(std::ptr::null_mut());
        let memory = CreateCompatibleDC(screen);
        let mut bits = std::ptr::null_mut();
        let bitmap = CreateDIBSection(
            memory,
            &info,
            DIB_RGB_COLORS,
            &mut bits,
            std::ptr::null_mut(),
            0,
        );
        let previous = SelectObject(memory, bitmap);
        BitBlt(
            memory,
            0,
            0,
            rect.width,
            rect.height,
            screen,
            rect.x,
            rect.y,
            SRCCOPY | CAPTUREBLT,
        );
        std::ptr::copy_nonoverlapping(bits.cast::<u8>(), pixels.as_mut_ptr(), pixels.len());
        SelectObject(memory, previous);
        DeleteObject(bitmap);
        DeleteDC(memory);
        ReleaseDC(std::ptr::null_mut(), screen);
    }
    pixels
}
