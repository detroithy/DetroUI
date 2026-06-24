use std::collections::HashMap;
use std::sync::Mutex;
use serde_json::{json, Value};

extern "system" {
    fn EnumWindows(lpEnumFunc: Option<unsafe extern "system" fn(isize, isize) -> i32>, lParam: isize) -> i32;
    fn GetWindowTextW(hWnd: isize, lpString: *mut u16, nMaxCount: i32) -> i32;
    fn GetClassNameW(hWnd: isize, lpString: *mut u16, nMaxCount: i32) -> i32;
    fn IsWindowVisible(hWnd: isize) -> i32;
    fn SetWindowPos(hWnd: isize, hWndInsertAfter: isize, X: i32, Y: i32, cx: i32, cy: i32, uFlags: u32) -> i32;
    fn ShowWindow(hWnd: isize, nCmdShow: i32) -> i32;
    fn GetWindowThreadProcessId(hWnd: isize, lpdwProcessId: *mut u32) -> u32;
    fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> isize;
    fn CloseHandle(hObject: isize) -> i32;
    fn GetModuleBaseNameW(hProcess: isize, hModule: isize, lpBaseName: *mut u16, nSize: u32) -> u32;
    fn GetWindowRect(hWnd: isize, lpRect: *mut RECT) -> i32;
    fn GetAsyncKeyState(nVirtKey: i32) -> i16;
    fn GetCursorPos(lpPoint: *mut POINT) -> i32;
    fn WindowFromPoint(point: POINT) -> isize;
    fn SystemParametersInfoW(uiAction: u32, uiParam: u32, pvParam: *mut std::ffi::c_void, fWinIni: u32) -> i32;
    fn SetWindowSubclass(hWnd: isize, pfnSubclass: Option<unsafe extern "system" fn(isize, u32, usize, isize, usize) -> isize>, uIdSubclass: usize, dwRefData: isize) -> i32;
    fn GetWindowLongW(hWnd: isize, nIndex: i32) -> i32;
    fn SetWindowLongW(hWnd: isize, nIndex: i32, dwNewLong: i32) -> i32;
    fn CallWindowProcW(lpPrevWndFunc: isize, hWnd: isize, Msg: u32, wParam: usize, lParam: isize) -> isize;
}

#[repr(C)]
struct RECT { left: i32, top: i32, right: i32, bottom: i32 }

#[repr(C)]
struct POINT { x: i32, y: i32 }

const PROCESS_QUERY_INFORMATION: u32 = 0x0400;
const PROCESS_VM_READ: u32 = 0x0010;

const SW_HIDE: i32 = 0;
const SW_SHOWNOACTIVATE: i32 = 4;
const SW_MINIMIZE: i32 = 6;
const SW_RESTORE: i32 = 9;
const SWP_NOACTIVATE: u32 = 0x0010;
const SWP_SHOWWINDOW: u32 = 0x0040;
const SWP_NOMOVE: u32 = 0x0002;
const SWP_NOSIZE: u32 = 0x0001;
const HWND_TOP: isize = 0;

// Windows Snap'i devre disbirakmak icin sabitler
const SPI_SETSNAPARRANGING: u32 = 0x00C4;
const SPIF_UPDATEINIFILE: u32 = 0x01;
const SPIF_SENDCHANGE: u32 = 0x02;
const GWL_WNDPROC: i32 = -4;
const WM_CONTEXTMENU: u32 = 0x007B;

// Virtual Desktop Manager
pub struct DesktopManager {
    pub desktops: Vec<Desktop>,
    pub current: usize,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Desktop {
    pub id: usize,
    pub name: String,
    pub shortcuts: Vec<Shortcut>,
    pub widget_ids: Vec<String>,
    #[serde(skip)]
    pub window_hwnds: Vec<isize>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Shortcut {
    pub name: String,
    pub path: String,
    pub icon: String,
    pub x: f64,
    pub y: f64,
}

impl DesktopManager {
    pub fn new() -> Self {
        DesktopManager {
            desktops: vec![
                Desktop { id: 0, name: "Ana".into(), shortcuts: vec![], widget_ids: vec!["clock".into(), "system-monitor".into()], window_hwnds: vec![] },
                Desktop { id: 1, name: "İş".into(), shortcuts: vec![], widget_ids: vec!["notes".into()], window_hwnds: vec![] },
                Desktop { id: 2, name: "Eğlence".into(), shortcuts: vec![], widget_ids: vec!["music-player".into(), "weather".into()], window_hwnds: vec![] },
            ],
            current: 0,
        }
    }

    pub fn switch_to(&mut self, id: usize, all_hwnds: &[isize], taskbar_h: i32) {
        if id >= self.desktops.len() || id == self.current { return; }
        let old = self.current;
        self.current = id;
        
        // OLD desktop: minimize
        for hwnd in &self.desktops[old].window_hwnds.clone() {
            if all_hwnds.contains(hwnd) {
                unsafe { ShowWindow(*hwnd, SW_MINIMIZE); }
            }
        }
        
        // NEW desktop: restore + tile
        for hwnd in &self.desktops[id].window_hwnds.clone() {
            if all_hwnds.contains(hwnd) {
                unsafe {
                    ShowWindow(*hwnd, SW_RESTORE);
                    ShowWindow(*hwnd, SW_SHOWNOACTIVATE);
                }
            }
        }
        
        // Clean dead HWNDs from ALL desktops
        for d in &mut self.desktops {
            d.window_hwnds.retain(|h| all_hwnds.contains(h));
        }
    }

    pub fn assign_window(&mut self, hwnd: isize, _title: &str) {
        // Auto-assign window to current desktop or create new entry
        if !self.desktops[self.current].window_hwnds.contains(&hwnd) {
            self.desktops[self.current].window_hwnds.push(hwnd);
        }
    }

    pub fn remove_window(&mut self, hwnd: isize) {
        for desktop in &mut self.desktops {
            desktop.window_hwnds.retain(|h| *h != hwnd);
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "current": self.current,
            "desktops": self.desktops,
        })
    }
}

// Window Manager - Windows API ile pencere listeleme
static mut ENUM_BUFFER: Vec<WindowInfo> = vec![];

#[derive(Clone)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub class: String,
    pub pid: u32,
    pub visible: bool,
}

unsafe extern "system" fn enum_proc(hwnd: isize, _lparam: isize) -> i32 {
    let mut buf = [0u16; 512];
    let mut class_buf = [0u16; 256];
    let mut pid: u32 = 0;

    let title_len = GetWindowTextW(hwnd, buf.as_mut_ptr(), 512);
    let class_len = GetClassNameW(hwnd, class_buf.as_mut_ptr(), 256);
    let visible = IsWindowVisible(hwnd);
    GetWindowThreadProcessId(hwnd, &mut pid);

    if title_len > 0 && visible != 0 {
        let title = String::from_utf16_lossy(&buf[..title_len as usize]);
        let class = String::from_utf16_lossy(&class_buf[..class_len as usize]);
        
        // Filter out detbar windows and system windows
        if !title.contains("detbar") && !class.contains("Windows.UI.Core") && class != "Progman" && class != "WorkerW" {
            ENUM_BUFFER.push(WindowInfo { hwnd, title, class, pid, visible: true });
        }
    }
    1
}

pub fn enum_windows() -> Vec<WindowInfo> {
    unsafe {
        ENUM_BUFFER.clear();
        EnumWindows(Some(enum_proc), 0);
        ENUM_BUFFER.clone()
    }
}

/// Pencereleri belli bir HWND listesi disinda gizle/goster
pub fn hide_all_windows_except(exclude_hwnds: &[isize]) {
    let all = enum_windows();
    for w in &all {
        if !exclude_hwnds.contains(&w.hwnd) {
            unsafe { ShowWindow(w.hwnd, SW_HIDE); }
        }
    }
}

pub fn show_window(hwnd: isize) {
    unsafe { ShowWindow(hwnd, SW_SHOWNOACTIVATE); }
}

pub fn hide_window(hwnd: isize) {
    unsafe { ShowWindow(hwnd, SW_HIDE); }
}

/// Tek pencere boyutlandir
pub fn set_window_bounds(hwnd: isize, x: i32, y: i32, w: i32, h: i32) {
    unsafe {
        SetWindowPos(hwnd, HWND_TOP, x, y, w, h, SWP_NOACTIVATE | SWP_SHOWWINDOW);
    }
}

/// Pencereyi one getir (boyut/konum degistirme)
pub fn bring_to_top(hwnd: isize) {
    unsafe {
        SetWindowPos(hwnd, HWND_TOP, 0, 0, 0, 0, SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW);
    }
}

/// Pencereyi HWND_BOTTOM'a gonder
pub fn send_to_bottom(hwnd: isize) {
    let hwnd_bottom: isize = 1;
    unsafe {
        SetWindowPos(hwnd, hwnd_bottom, 0, 0, 0, 0, SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE);
    }
}

/// Komorebi tarzi master-stack tiling - ilk pencere master
pub fn tile_desktop_windows(hwnds: &[isize], taskbar_height: i32) {
    if hwnds.is_empty() { return; }

    let screen = get_screen_work_area(taskbar_height);
    let n = hwnds.len();

    if n == 1 {
        set_window_bounds(hwnds[0], screen.0, screen.1, screen.2, screen.3);
    } else {
        let master_w = (screen.2 as f64 * 0.6) as i32;
        let stack_w = screen.2 - master_w;
        let stack_h = if n > 2 { screen.3 / ((n - 1) as i32).max(1) } else { screen.3 };

        set_window_bounds(hwnds[0], screen.0, screen.1, master_w, screen.3);

        for i in 1..n {
            let row = (i - 1) as i32;
            let y = screen.1 + row * stack_h;
            let h = if i >= n - 1 { screen.1 + screen.3 - y } else { stack_h };
            set_window_bounds(hwnds[i], screen.0 + master_w, y, stack_w, h.max(100));
        }
    }
}

/// Son odaklanan pencereyi listenin basina al (master yap)
pub fn promote_to_master(desktop_list: &mut Vec<isize>, hwnd: isize) {
    if let Some(pos) = desktop_list.iter().position(|h| *h == hwnd) {
        if pos > 0 {
            desktop_list.remove(pos);
            desktop_list.insert(0, hwnd);
        }
    }
}

/// Ekran calisma alanini al (taskbar haric)
fn get_screen_work_area(taskbar_height: i32) -> (i32, i32, i32, i32) {
    let (sw, sh) = get_screen_size();
    (0, 0, sw, sh - taskbar_height)
}

fn get_screen_size() -> (i32, i32) {
    // Default 1920x1080
    (1920, 1080)
}

/// Pencere process adini al (exe adi, kucuk harf)
pub fn get_process_name(hwnd: isize) -> Option<String> {
    unsafe {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 { return None; }
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
        if handle == 0 { return None; }
        let mut buf = [0u16; 260];
        let len = GetModuleBaseNameW(handle, 0, buf.as_mut_ptr(), 260);
        CloseHandle(handle);
        if len > 0 {
            Some(String::from_utf16_lossy(&buf[..len as usize]).to_lowercase())
        } else {
            None
        }
    }
}

static mut EDGE_TRANSFER_ENABLED: bool = true;

/// Windows Snap ozelligini devre disi birak
pub fn disable_windows_snap() {
    unsafe {
        let mut disable: i32 = 0; // FALSE = disable
        SystemParametersInfoW(
            SPI_SETSNAPARRANGING,
            0,
            &mut disable as *mut i32 as *mut std::ffi::c_void,
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        );
    }
}

/// Edge transfer: fare pozisyonuna bakarak masaustu degistir
pub fn check_edge_transfer(screen_w: i32, _screen_h: i32, _taskbar_h: i32) -> Vec<(isize, i32)> {
    let mut transfers = Vec::new();
    let threshold = 2; // Kenardan 2px icinde tetikle
    
    // Sadece sol fare tusu basiliyken kontrol et
    let dragging = unsafe { (GetAsyncKeyState(1) as u32) & 0x8000 != 0 };
    if !dragging { return transfers; }
    
    // Fare pozisyonunu al
    let mut cursor = POINT { x: 0, y: 0 };
    if unsafe { GetCursorPos(&mut cursor) } == 0 { return transfers; }
    
    // Fare sag kenarda -> sonraki masaustu
    if cursor.x >= screen_w - threshold {
        let wins = enum_windows();
        for w in &wins {
            if w.title.contains("detbar") || w.title.is_empty() { continue; }
            if w.class.contains("WorkerW") || w.class.contains("Progman") || w.class.contains("Shell") { continue; }
            let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            unsafe { GetWindowRect(w.hwnd, &mut rect); }
            if rect.right <= 0 || rect.bottom <= 0 { continue; }
            transfers.push((w.hwnd, 1));
            break;
        }
    }
    // Fare sol kenarda -> onceki masaustu
    else if cursor.x <= threshold {
        let wins = enum_windows();
        for w in &wins {
            if w.title.contains("detbar") || w.title.is_empty() { continue; }
            if w.class.contains("WorkerW") || w.class.contains("Progman") || w.class.contains("Shell") { continue; }
            let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            unsafe { GetWindowRect(w.hwnd, &mut rect); }
            if rect.right <= 0 || rect.bottom <= 0 { continue; }
            transfers.push((w.hwnd, -1));
            break;
        }
    }
    
    transfers
}

/// Fare imlecinin altindaki pencereyi bul
pub fn get_window_under_cursor() -> Option<WindowInfo> {
    let mut pt = POINT { x: 0, y: 0 };
    unsafe {
        if GetCursorPos(&mut pt) == 0 { return None; }
        let hwnd = WindowFromPoint(pt);
        if hwnd == 0 { return None; }
        
        let mut buf = [0u16; 512];
        let mut class_buf = [0u16; 256];
        let title_len = GetWindowTextW(hwnd, buf.as_mut_ptr(), 512);
        let class_len = GetClassNameW(hwnd, class_buf.as_mut_ptr(), 256);
        let visible = IsWindowVisible(hwnd);
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        
        if visible == 0 { return None; }
        
        let title = if title_len > 0 {
            String::from_utf16_lossy(&buf[..title_len as usize])
        } else { String::new() };
        let class = if class_len > 0 {
            String::from_utf16_lossy(&class_buf[..class_len as usize])
        } else { String::new() };
        
        Some(WindowInfo { hwnd, title, class, pid, visible: true })
    }
}
