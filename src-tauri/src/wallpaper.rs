use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::Arc;
use std::thread;
use std::io::{Read, Seek, SeekFrom};

static SERVER_RUNNING: AtomicBool = AtomicBool::new(false);
static CURRENT_PORT: AtomicU16 = AtomicU16::new(0);
static mut SERVER_RUN_FLAG: Option<Arc<AtomicBool>> = None;

extern "system" {
    fn FindWindowW(lpClassName: *const u16, lpWindowName: *const u16) -> isize;
    fn FindWindowExW(hWndParent: isize, hWndChildAfter: isize, lpszClass: *const u16, lpszWindow: *const u16) -> isize;
    fn SendMessageW(hWnd: isize, Msg: u32, wParam: usize, lParam: isize) -> isize;
    fn SetParent(hWndChild: isize, hWndNewParent: isize) -> isize;
    fn SetWindowPos(hWnd: isize, hWndInsertAfter: isize, X: i32, Y: i32, cx: i32, cy: i32, uFlags: u32) -> i32;
    fn SHAppBarMessage(dwMessage: u32, pData: *mut APPBARDATA) -> isize;
    fn SetWindowLongPtrW(hWnd: isize, nIndex: i32, dwNewLong: isize) -> isize;
    fn CallWindowProcW(lpPrevWndFunc: isize, hWnd: isize, Msg: u32, wParam: usize, lParam: isize) -> isize;
    fn GetWindowLongPtrW(hWnd: isize, nIndex: i32) -> isize;
}

#[repr(C)]
struct APPBARDATA {
    cbSize: u32,
    hWnd: isize,
    uCallbackMessage: u32,
    uEdge: u32,
    rc: RECT,
    lParam: isize,
}

#[repr(C)]
struct RECT { left: i32, top: i32, right: i32, bottom: i32 }

const HWND_BOTTOM: isize = 1;
const HWND_NOTOPMOST: isize = -2;
const SWP_NOMOVE: u32 = 0x0002;
const SWP_NOSIZE: u32 = 0x0001;
const SWP_NOACTIVATE: u32 = 0x0010;
const WM_SPAWN_WORKER: u32 = 0x052C;
const ABM_SETSTATE: u32 = 0x0000000a;
const ABS_AUTOHIDE: u32 = 0x00000001;

fn to_u16(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn embed_behind_desktop(wallpaper_hwnd: isize) -> Result<(), String> {
    unsafe {
        if wallpaper_hwnd == 0 { return Err("gecersiz HWND".into()); }

        let progman_s = to_u16("Progman");
        let progman = FindWindowW(progman_s.as_ptr(), std::ptr::null());
        if progman == 0 { return Err("Progman bulunamadi".into()); }

        // WorkerW olusturulana kadar bekle (5 deneme, 200ms aralikla)
        let workerw_s = to_u16("WorkerW");
        let defview_s = to_u16("SHELLDLL_DefView");
        let mut workerw_hwnd: isize = 0;

        for attempt in 0..5 {
            SendMessageW(progman, WM_SPAWN_WORKER, 0, 0);
            thread::sleep(std::time::Duration::from_millis(200));

            let mut hwnd: isize = 0;
            loop {
                hwnd = FindWindowExW(progman, hwnd, workerw_s.as_ptr(), std::ptr::null());
                if hwnd == 0 { break; }
                let dv = FindWindowExW(hwnd, 0, defview_s.as_ptr(), std::ptr::null());
                if dv != 0 { workerw_hwnd = hwnd; break; }
            }
            if workerw_hwnd != 0 { break; }
            eprintln!("[wallpaper] WorkerW deneme {}/5 basarisiz", attempt + 1);
        }

        if workerw_hwnd == 0 { return Err("WorkerW bulunamadi (5 deneme)".into()); }

        SetParent(wallpaper_hwnd, workerw_hwnd);
        SetWindowPos(wallpaper_hwnd, HWND_BOTTOM, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
        Ok(())
    }
}

/// Video server'i durdur
pub fn stop_video_server() {
    unsafe {
        if let Some(flag) = &SERVER_RUN_FLAG {
            flag.store(false, Ordering::SeqCst);
        }
    }
    SERVER_RUNNING.store(false, Ordering::SeqCst);
    CURRENT_PORT.store(0, Ordering::SeqCst);
}

/// Video dosyasini serve eden minimal HTTP server
pub fn start_video_server(file_path: String) -> Result<u16, String> {
    // Eski server varsa durdur
    if SERVER_RUNNING.load(Ordering::SeqCst) {
        stop_video_server();
        thread::sleep(std::time::Duration::from_millis(200));
    }

    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("Bind: {}", e))?;
    let port = listener.local_addr().map_err(|e| format!("Port: {}", e))?.port();

    listener.set_nonblocking(true).map_err(|e| format!("NonBlocking: {}", e))?;

    CURRENT_PORT.store(port, Ordering::SeqCst);
    SERVER_RUNNING.store(true, Ordering::SeqCst);
    let run = Arc::new(AtomicBool::new(true));

    unsafe {
        SERVER_RUN_FLAG = Some(run.clone());
    }

    let path = file_path.clone();
    thread::spawn(move || {
        let file_size = std::fs::metadata(&path).ok().map(|m| m.len()).unwrap_or(0);
        let ext = path.rsplit('.').next().unwrap_or("mp4").to_lowercase();
        let mime = match ext.as_str() {
            "webm" => "video/webm", "avi" => "video/x-msvideo",
            "mkv" => "video/x-matroska", "mov" => "video/quicktime",
            _ => "video/mp4",
        };

        loop {
            if !run.load(Ordering::SeqCst) { break; }
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut buf = [0u8; 4096];
                    let n = match stream.read(&mut buf) {
                        Ok(n) => n, Err(_) => continue,
                    };
                    if n == 0 { continue; }
                    let request = String::from_utf8_lossy(&buf[..n.min(2000)]);
                    let is_range = request.contains("Range: bytes=");
                    let range_start = if is_range {
                        request.split("Range: bytes=").nth(1)
                            .and_then(|s| s.split('-').next())
                            .and_then(|s| s.trim().parse::<u64>().ok()).unwrap_or(0)
                    } else { 0 };
                    let status = if is_range { "206 Partial Content" } else { "200 OK" };
                    let content_len = file_size.saturating_sub(range_start);
                    let range_hdr = if is_range {
                        format!("\r\nContent-Range: bytes {}-{}/{}", range_start, file_size.saturating_sub(1), file_size)
                    } else { String::new() };
                    let header = format!(
                        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccept-Ranges: bytes{}\r\nConnection: close\r\n\r\n",
                        status, mime, content_len, range_hdr
                    );
                    if std::io::Write::write_all(&mut stream, header.as_bytes()).is_err() { continue; }
                    if let Ok(file) = std::fs::File::open(&path) {
                        let mut file = file;
                        if range_start > 0 { let _ = file.seek(SeekFrom::Start(range_start)); }
                        let mut fbuf = [0u8; 65536];
                        loop {
                            match file.read(&mut fbuf) {
                                Ok(0) => break,
                                Ok(n) => { if std::io::Write::write_all(&mut stream, &fbuf[..n]).is_err() { break; } }
                                Err(_) => break,
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
                Err(_) => break,
            }
        }
        SERVER_RUNNING.store(false, Ordering::SeqCst);
        CURRENT_PORT.store(0, Ordering::SeqCst);
    });

    Ok(port)
}

/// Pencereyi HWND_BOTTOM yap
pub fn set_bottom_zindex(hwnd: isize) -> Result<(), String> {
    unsafe {
        if hwnd == 0 { return Err("gecersiz HWND".into()); }
        SetWindowPos(hwnd, HWND_NOTOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
        SetWindowPos(hwnd, HWND_BOTTOM, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
        Ok(())
    }
}

/// Windows gorev cubugunu otomatik gizle
pub fn auto_hide_taskbar() {
    unsafe {
        let tray = to_u16("Shell_TrayWnd");
        let hwnd = FindWindowW(tray.as_ptr(), std::ptr::null());
        if hwnd == 0 { return; }

        let mut abd = APPBARDATA {
            cbSize: std::mem::size_of::<APPBARDATA>() as u32,
            hWnd: hwnd,
            uCallbackMessage: 0,
            uEdge: 0,
            rc: RECT { left: 0, top: 0, right: 0, bottom: 0 },
            lParam: ABS_AUTOHIDE as isize,
        };

        SHAppBarMessage(ABM_SETSTATE, &mut abd);
    }
}

// WebView2 child penceresini subclassla, context menu'yu engelle
static mut OLD_WNDPROC: isize = 0;
const GWL_WNDPROC: i32 = -4;
const WM_CONTEXTMENU: u32 = 0x007B;

unsafe extern "system" fn subclass_proc(hwnd: isize, msg: u32, wp: usize, lp: isize) -> isize {
    if msg == WM_CONTEXTMENU {
        return 0;
    }
    CallWindowProcW(OLD_WNDPROC, hwnd, msg, wp, lp)
}

pub fn disable_context_menu(parent_hwnd: isize) {
    unsafe {
        let class = to_u16("Chrome_WidgetWin_1");
        let mut child = FindWindowExW(parent_hwnd, 0, class.as_ptr(), std::ptr::null());
        if child == 0 {
            let class0 = to_u16("Chrome_WidgetWin_0");
            child = FindWindowExW(parent_hwnd, 0, class0.as_ptr(), std::ptr::null());
        }
        if child != 0 {
            OLD_WNDPROC = SetWindowLongPtrW(child, GWL_WNDPROC, subclass_proc as isize);
        }
    }
}

/// Pencereyi focus almayan yap (WS_EX_NOACTIVATE)
pub fn make_noactivate(hwnd: isize) {
    let gwl_exstyle: i32 = -20;
    let ws_ex_noactivate: isize = 0x08000000;
    unsafe {
        let current = GetWindowLongPtrW(hwnd, gwl_exstyle);
        SetWindowLongPtrW(hwnd, gwl_exstyle, current | ws_ex_noactivate);
    }
}
