use crate::AppState;
use serde_json::{json, Value};
use std::process::Command;
use std::sync::Mutex;
use sysinfo::System;
use tauri::{State, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

// ===== Sistem Bilgileri =====

#[tauri::command]
pub async fn get_cpu_load() -> Result<Value, String> {
    let mut sys = System::new_all();
    sys.refresh_cpu_all();
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_cpu_all();

    let cpus: Vec<Value> = sys.cpus().iter().map(|cpu| {
        json!({
            "load": cpu.cpu_usage() as f64,
            "user": cpu.cpu_usage() as f64,
            "system": 0.0
        })
    }).collect();

    Ok(json!({
        "currentLoad": sys.global_cpu_usage() as f64,
        "cpus": cpus
    }))
}

#[tauri::command]
pub async fn get_memory() -> Result<Value, String> {
    let mut sys = System::new_all();
    sys.refresh_memory();
    Ok(json!({
        "total": sys.total_memory(),
        "free": sys.available_memory(),
        "used": sys.used_memory(),
        "usedPercent": (sys.used_memory() as f64 / sys.total_memory() as f64 * 100.0).round()
    }))
}

#[tauri::command]
pub async fn get_disk() -> Result<Value, String> {
    Ok(json!([]))
}

#[tauri::command]
pub async fn get_battery() -> Result<Value, String> {
    Ok(json!({
        "level": 100,
        "charging": false,
        "timeRemaining": null,
        "type": null
    }))
}

#[tauri::command]
pub async fn get_cpu_temp() -> Result<Value, String> {
    Ok(json!({
        "main": 0,
        "max": 0,
        "cores": []
    }))
}

#[tauri::command]
pub async fn get_network() -> Result<Value, String> {
    Ok(json!({
        "stats": [],
        "interfaces": []
    }))
}

// ===== Hava Durumu =====

#[tauri::command]
pub async fn get_weather(city: String) -> Result<Value, String> {
    let coords = match city.to_lowercase().as_str() {
        "ankara" => (39.9334, 32.8597),
        "izmir" => (38.4192, 27.1287),
        "london" => (51.5074, -0.1278),
        "new york" => (40.7128, -74.006),
        "berlin" => (52.52, 13.405),
        _ => (41.0082, 28.9784),
    };

    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,weather_code,wind_speed_10m,apparent_temperature&daily=weather_code,temperature_2m_max,temperature_2m_min&timezone=auto&forecast_days=5",
        coords.0, coords.1
    );

    let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    let data: Value = resp.json().await.map_err(|e| e.to_string())?;

    let weather_codes: std::collections::HashMap<i64, (&str, &str)> = [
        (0, ("GÃ¼neÅŸli", "â˜€ï¸")),
        (1, ("Az Bulutlu", "ğŸŒ¤ï¸")),
        (2, ("ParÃ§alÄ± Bulutlu", "â›…")),
        (3, ("KapalÄ±", "â˜ï¸")),
        (45, ("Sisli", "ğŸŒ«ï¸")),
        (51, ("Hafif Ã‡iseleme", "ğŸŒ¦ï¸")),
        (61, ("Hafif YaÄŸmur", "ğŸŒ§ï¸")),
        (63, ("Orta YaÄŸmur", "ğŸŒ§ï¸")),
        (65, ("Åiddetli YaÄŸmur", "ğŸŒ§ï¸")),
        (71, ("Hafif Kar", "â„ï¸")),
        (73, ("Orta Kar", "â„ï¸")),
        (75, ("YoÄŸun Kar", "â„ï¸")),
        (95, ("FÄ±rtÄ±na", "â›ˆï¸")),
    ].iter().cloned().collect();

    let current = &data["current"];
    let weather_code = current["weather_code"].as_i64().unwrap_or(0);
    let (desc, icon) = weather_codes.get(&weather_code).unwrap_or(&("Bilinmiyor", "â“"));

    let daily = &data["daily"];
    let days: Vec<Value> = (0..5).filter_map(|i| {
        Some(json!({
            "date": format!("Day {}", i),
            "max": daily["temperature_2m_max"][i].as_f64().unwrap_or(0.0),
            "min": daily["temperature_2m_min"][i].as_f64().unwrap_or(0.0),
            "weatherCode": daily["weather_code"][i].as_i64().unwrap_or(0),
            "description": desc,
            "icon": icon
        }))
    }).collect();

    Ok(json!({
        "current": {
            "temperature": current["temperature_2m"].as_f64().unwrap_or(0.0),
            "humidity": current["relative_humidity_2m"].as_f64().unwrap_or(0.0),
            "weatherCode": weather_code,
            "description": desc,
            "icon": icon,
            "windSpeed": current["wind_speed_10m"].as_f64().unwrap_or(0.0),
            "feelsLike": current["apparent_temperature"].as_f64().unwrap_or(0.0)
        },
        "daily": days,
        "city": city
    }))
}

// ===== Uygulamalar =====

#[tauri::command]
pub async fn get_installed_apps() -> Result<Vec<Value>, String> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", r#"
            $seen = @{}
            $apps = @()
            
            # Registry'den uygulamalari al
            $paths = @(
                'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*',
                'HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*',
                'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*'
            )
            foreach ($path in $paths) {
                if (Test-Path $path) {
                    Get-ItemProperty $path | Where-Object {$_.DisplayName} | ForEach-Object {
                        $key = $_.DisplayName.ToLower()
                        if (-not $seen.ContainsKey($key)) {
                            $seen[$key] = $true
                            $loc = $_.InstallLocation
                            $exePath = ''
                            # InstallLocation bir klasorse icindeki .exe'yi bul
                            if ($loc -and (Test-Path $loc) -and (Get-Item $loc).PSIsContainer) {
                                $exes = Get-ChildItem -Path $loc -Filter '*.exe' -ErrorAction SilentlyContinue | Select-Object -First 1
                                if ($exes) { $exePath = $exes.FullName }
                            } elseif ($loc -and $loc.EndsWith('.exe') -and (Test-Path $loc)) {
                                $exePath = $loc
                            }
                            # DisplayIcon'dan da dene
                            if (-not $exePath -and $_.DisplayIcon) {
                                $ico = $_.DisplayIcon -replace ',.*',''
                                if ($ico.EndsWith('.exe') -and (Test-Path $ico)) { $exePath = $ico }
                            }
                            $apps += @{
                                name = $_.DisplayName
                                version = $_.DisplayVersion
                                publisher = $_.Publisher
                                path = if ($exePath) { $exePath } else { $loc }
                                source = 'registry'
                            }
                        }
                    }
                }
            }
            
            # Start Menu'den .lnk dosyalarini tara
            $startMenuPaths = @(
                "$env:ProgramData\Microsoft\Windows\Start Menu\Programs",
                "$env:APPDATA\Microsoft\Windows\Start Menu\Programs"
            )
            foreach ($smPath in $startMenuPaths) {
                if (Test-Path $smPath) {
                    Get-ChildItem -Path $smPath -Recurse -Filter '*.lnk' -ErrorAction SilentlyContinue | ForEach-Object {
                        $name = $_.BaseName
                        $key = $name.ToLower()
                        if (-not $seen.ContainsKey($key)) {
                            $seen[$key] = $true
                            $apps += @{
                                name = $name
                                version = ''
                                publisher = ''
                                path = $_.FullName
                                source = 'startmenu'
                            }
                        }
                    }
                }
            }
            
            # PATH'teki .exe dosyalarini tara
            $env:Path -split ';' | Where-Object { Test-Path $_ } | ForEach-Object {
                Get-ChildItem -Path $_ -Filter '*.exe' -ErrorAction SilentlyContinue | Select-Object -First 50 | ForEach-Object {
                    $name = $_.BaseName
                    $key = $name.ToLower()
                    if (-not $seen.ContainsKey($key)) {
                        $seen[$key] = $true
                        $apps += @{
                            name = $name
                            version = ''
                            publisher = ''
                            path = $_.FullName
                            source = 'path'
                        }
                    }
                }
            }
            
            $apps | ConvertTo-Json -Compress
        "#])
        .output()
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let apps: Vec<Value> = serde_json::from_str(&stdout).unwrap_or_default();
    Ok(apps.into_iter().filter(|a| a.get("name").is_some()).collect())
}

#[tauri::command]
pub async fn launch_app(app: tauri::AppHandle, state: State<'_, AppState>, path: String) -> Result<(), String> {
    // URI protokolleri ve özel komutlar
    let is_uri = path.starts_with("msedge:") || path.starts_with("chrome:") || path.starts_with("firefox:") || path.starts_with("http://") || path.starts_with("https://");
    let is_settings = path.starts_with("ms-settings:");
    let is_syscmd = path.starts_with("shutdown") || path.starts_with("rundll32");
    let is_folder = std::path::Path::new(&path).is_dir();
    let is_exe = path.ends_with(".exe");

    if is_uri || is_settings || is_syscmd {
        Command::new("cmd")
            .args(["/C", "start", "", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    } else if is_exe {
        // .exe dosyasini dogrudan calistir, cmd /C kullanma
        if path.contains('/') || path.contains('\\') {
            // Tam yol varsa dogrudan calistir
            Command::new(&path)
                .spawn()
                .map_err(|e| format!("{} açılamadı: {}", path, e))?;
        } else {
            // Sadece exe adi varsa start ile dene
            Command::new("cmd")
                .args(["/C", "start", "", &path])
                .spawn()
                .map_err(|e| format!("{} açılamadı: {}", path, e))?;
        }
    } else if is_folder {
        Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("{} açılamadı: {}", path, e))?;
    } else if path.ends_with(".lnk") {
        Command::new("cmd")
            .args(["/C", "start", "", &path])
            .spawn()
            .map_err(|e| format!("{} açılamadı: {}", path, e))?;
    } else {
        // Diğer her şey için start kullan
        Command::new("cmd")
            .args(["/C", "start", "", &path])
            .spawn()
            .map_err(|e| format!("{} açılamadı: {}", path, e))?;
    }

    // 1.5 saniye bekle, yeni pencereleri bul ve anlik masaustune ata
    drop(state);

    let handle = app.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        let wins = crate::virtual_desktop::enum_windows();
        let new_hwnds: Vec<isize> = wins.iter()
            .filter(|w| !w.title.contains("detbar") && !w.title.is_empty()
                && !w.class.contains("WorkerW") && !w.class.contains("Progman"))
            .map(|w| w.hwnd).collect();
        
        if !new_hwnds.is_empty() {
            let state: State<'_, AppState> = handle.state();
            let (auto_assign, app_map, taskbar_h2) = {
                let store = match state.store.lock() { Ok(s) => s, Err(_) => return };
                let auto_assign = store.get("autoAssignWindows").as_bool().unwrap_or(true);
                let app_map = store.get("appDesktopMap").as_object().cloned().unwrap_or_default();
                let taskbar_h2 = store.get("taskbarHeight").as_i64().unwrap_or(48) as i32;
                drop(store);
                (auto_assign, app_map, taskbar_h2)
            };
            
            let mut mgr = match state.desktop_manager.lock() { Ok(m) => m, Err(_) => return };
            let cur = mgr.current;
            let existing: Vec<isize> = mgr.desktops.iter()
                .flat_map(|d| d.window_hwnds.iter().cloned())
                .collect();
            
            for hwnd in &new_hwnds {
                if existing.contains(hwnd) { continue; }
                let proc_name = crate::virtual_desktop::get_process_name(*hwnd);
                let target_desk = proc_name.as_ref()
                    .and_then(|name| app_map.get(name))
                    .and_then(|v| v.as_i64())
                    .map(|id| id as usize)
                    .filter(|id| *id < mgr.desktops.len());
                
                if let Some(desk_id) = target_desk {
                    mgr.desktops[desk_id].window_hwnds.push(*hwnd);
                } else if auto_assign {
                    mgr.desktops[cur].window_hwnds.push(*hwnd);
                }
            }
            
            let cur_hwnds = mgr.desktops[cur].window_hwnds.clone();
            drop(mgr);
            crate::virtual_desktop::tile_desktop_windows(&cur_hwnds, taskbar_h2);
        }
    });

    Ok(())
}

// ===== Medya =====

#[tauri::command]
pub async fn get_media_sessions() -> Result<Vec<Value>, String> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", r#"
            Add-Type @"
              using System;
              using System.Runtime.InteropServices;
              public class WinIcon {
                [DllImport("shell32.dll")] public static extern IntPtr ExtractIcon(IntPtr hInst, string file, int index);
              }
"@
            $sessions = @()
            Get-Process | Where-Object {
              $_.MainWindowTitle -ne "" -and (
                $_.ProcessName -match 'spotify|vlc|music|player|media|groove|aimp|foobar|winamp|itunes|potplayer|mpc' -or
                $_.MainWindowTitle -match ' - |â€“|â€”'
              )
            } | ForEach-Object {
              $title = $_.MainWindowTitle
              $artist = ""
              $songTitle = $title
              if ($title -match '^(.+?)\s*[â€“â€”-]\s+(.+)$') {
                $artist = $matches[1].Trim()
                $songTitle = $matches[2].Trim()
              }
              $iconPath = $_.Path
              $iconB64 = ""
              if ($iconPath -and (Test-Path $iconPath)) {
                try {
                  $hIcon = [WinIcon]::ExtractIcon([IntPtr]::Zero, $iconPath, 0)
                  if ($hIcon -ne [IntPtr]::Zero -and $hIcon -ne [IntPtr]::new(-1)) {
                    $bmp = [System.Drawing.Icon]::FromHandle($hIcon).ToBitmap()
                    $ms = New-Object System.IO.MemoryStream
                    $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
                    $iconB64 = [Convert]::ToBase64String($ms.ToArray())
                    $ms.Dispose(); $bmp.Dispose()
                  }
                } catch {}
              }
              $sessions += @{
                appName = $_.ProcessName
                title = $songTitle
                artist = $artist
                fullTitle = $title
                icon = $iconB64
                playbackStatus = "Playing"
              }
            }
            $sessions | ConvertTo-Json -Compress
        "#])
        .output()
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let sessions: Vec<Value> = serde_json::from_str(&stdout).unwrap_or_default();
    Ok(sessions.into_iter().filter(|s| s.get("appName").is_some()).collect())
}

fn send_media_key(key: &str) {
    let script = format!(
        "$obj = New-Object -ComObject WScript.Shell; $obj.SendKeys(\"{}\")",
        key
    );
    let _ = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .spawn();
}

#[tauri::command]
pub async fn media_play(_app: String) -> Result<(), String> {
    send_media_key("[{MEDIA_PLAY_PAUSE}]");
    Ok(())
}

#[tauri::command]
pub async fn media_pause(_app: String) -> Result<(), String> {
    send_media_key("[{MEDIA_PLAY_PAUSE}]");
    Ok(())
}

#[tauri::command]
pub async fn media_next(_app: String) -> Result<(), String> {
    send_media_key("[{MEDIA_NEXT_TRACK}]");
    Ok(())
}

#[tauri::command]
pub async fn media_prev(_app: String) -> Result<(), String> {
    send_media_key("[{MEDIA_PREV_TRACK}]");
    Ok(())
}

#[tauri::command]
pub async fn media_toggle() -> Result<(), String> {
    send_media_key("[{MEDIA_PLAY_PAUSE}]");
    Ok(())
}

#[tauri::command]
pub async fn media_seek(percent: f64) -> Result<(), String> {
    let key = if percent > 0.5 { "^({RIGHT})" } else { "^({LEFT})" };
    send_media_key(key);
    Ok(())
}

#[tauri::command]
pub async fn media_volume_up() -> Result<(), String> {
    send_media_key("{VOLUME_UP}");
    Ok(())
}

#[tauri::command]
pub async fn media_volume_down() -> Result<(), String> {
    send_media_key("{VOLUME_DOWN}");
    Ok(())
}

#[tauri::command]
pub async fn media_mute() -> Result<(), String> {
    send_media_key("{VOLUME_MUTE}");
    Ok(())
}

// ===== Tema =====

#[tauri::command]
pub async fn get_theme(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.store.lock().unwrap().get("theme").as_str().unwrap_or("dark").to_string())
}

#[tauri::command]
pub async fn set_theme(state: State<'_, AppState>, theme: String, app: tauri::AppHandle) -> Result<bool, String> {
    state.store.lock().unwrap().set("theme", json!(theme));
    let accent = state.store.lock().unwrap().get("accentColor").as_str().unwrap_or("#00d4ff").to_string();
    let _ = app.emit("theme-changed", json!({ "theme": theme, "accentColor": accent }));
    Ok(true)
}

#[tauri::command]
pub async fn get_accent_color(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.store.lock().unwrap().get("accentColor").as_str().unwrap_or("#00d4ff").to_string())
}

#[tauri::command]
pub async fn set_accent_color(state: State<'_, AppState>, color: String, app: tauri::AppHandle) -> Result<bool, String> {
    state.store.lock().unwrap().set("accentColor", json!(color));
    let theme = state.store.lock().unwrap().get("theme").as_str().unwrap_or("dark").to_string();
    let _ = app.emit("theme-changed", json!({ "theme": theme, "accentColor": color }));
    Ok(true)
}

// ===== Store =====

#[tauri::command]
pub async fn get_store_value(state: State<'_, AppState>, key: String) -> Result<Value, String> {
    Ok(state.store.lock().unwrap().get(&key))
}

#[tauri::command]
pub async fn set_store_value(state: State<'_, AppState>, key: String, value: Value, app: tauri::AppHandle) -> Result<bool, String> {
    state.store.lock().unwrap().set(&key, value.clone());
    match key.as_str() {
        "taskbarPosition" | "taskbarHeight" => {
            let _ = app.emit("taskbar-settings-changed", json!({
                "position": state.store.lock().unwrap().get("taskbarPosition").as_str().unwrap_or("bottom"),
                "height": state.store.lock().unwrap().get("taskbarHeight").as_i64().unwrap_or(48)
            }));
        }
        k if k.starts_with("widget_") && k.ends_with("_visible") => {
            let widget_id = k.trim_start_matches("widget_").trim_end_matches("_visible");
            let _ = app.emit("widget-config-changed", json!({ "id": widget_id, "visible": value }));
        }
        _ => {}
    }
    Ok(true)
}

// ===== Notlar =====

#[tauri::command]
pub async fn get_notes(state: State<'_, AppState>) -> Result<Value, String> {
    Ok(state.store.lock().unwrap().get("notes"))
}

#[tauri::command]
pub async fn save_notes(state: State<'_, AppState>, notes: Value) -> Result<bool, String> {
    state.store.lock().unwrap().set("notes", notes);
    Ok(true)
}

// ===== Ekran =====

#[tauri::command]
pub async fn screen_information() -> Result<Value, String> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", r#"
            Add-Type @"
              using System;
              using System.Runtime.InteropServices;
              public class ScreenInfo {
                [DllImport("user32.dll")] public static extern IntPtr GetDesktopWindow();
                [DllImport("user32.dll")] public static extern IntPtr MonitorFromWindow(IntPtr hwnd, uint flags);
                [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern bool GetMonitorInfo(IntPtr hMonitor, ref MONITORINFOEX lpmi);
                [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
                public struct RECT { public int Left, Top, Right, Bottom; }
                [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
                public struct MONITORINFOEX {
                  public int cbSize;
                  public RECT rcMonitor;
                  public RECT rcWork;
                  public uint dwFlags;
                  [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)]
                  public string szDevice;
                }
              }
"@
            $mi = New-Object ScreenInfo+MONITORINFOEX
            $mi.cbSize = [System.Runtime.InteropServices.Marshal]::SizeOf($mi)
            $desktop = [ScreenInfo]::GetDesktopWindow()
            $monitor = [ScreenInfo]::MonitorFromWindow($desktop, 1)
            [ScreenInfo]::GetMonitorInfo($monitor, [ref]$mi) | Out-Null
            $mi.rcMonitor | ConvertTo-Json -Compress
        "#])
        .output()
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let rect: Value = serde_json::from_str(stdout.trim()).unwrap_or(json!({}));

    let left = rect["Left"].as_i64().unwrap_or(0);
    let top = rect["Top"].as_i64().unwrap_or(0);
    let right = rect["Right"].as_i64().unwrap_or(1920);
    let bottom = rect["Bottom"].as_i64().unwrap_or(1080);

    Ok(json!({
        "width": right - left,
        "height": bottom - top,
        "workArea": { "x": left, "y": top, "width": right - left, "height": bottom - top },
        "scaleFactor": 1.0
    }))
}

#[tauri::command]
pub async fn pick_file(title: String, filter: String) -> Result<Option<String>, String> {
    // -NonInteractive kaldirildi - GUI dialog icin gerekli
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &format!(
            r#"Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.OpenFileDialog; $f.Title = '{}'; $f.Filter = '{}'; if ($f.ShowDialog() -eq 'OK') {{ $f.FileName }} else {{ "" }}"#,
            title, filter
        )])
        .output()
        .map_err(|e| e.to_string())?;

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() { Ok(None) } else { Ok(Some(path)) }
}

#[tauri::command]
pub async fn pick_folder(title: String) -> Result<Option<String>, String> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &format!(
            r#"Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.FolderBrowserDialog; $f.Description = '{}'; $f.ShowNewFolderButton = $true; if ($f.ShowDialog() -eq 'OK') {{ $f.SelectedPath }} else {{ "" }}"#,
            title
        )])
        .output()
        .map_err(|e| e.to_string())?;

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() { Ok(None) } else { Ok(Some(path)) }
}

// ===== Widget Pencere YÃ¶netimi =====

fn widget_config(id: &str) -> Option<(u32, u32, &'static str)> {
    match id {
        "clock" => Some((250, 250, "widgets/clock/index.html")),
        "system-monitor" => Some((300, 350, "widgets/system-monitor/index.html")),
        "weather" => Some((280, 320, "widgets/weather/index.html")),
        "notes" => Some((350, 400, "widgets/notes/index.html")),
        "music-player" => Some((380, 280, "widgets/music-player/index.html")),
        _ => None,
    }
}

#[tauri::command]
pub async fn create_widget(id: String, app: tauri::AppHandle, state: State<'_, AppState>) -> Result<bool, String> {
    let label = format!("widget-{}", id);
    if app.get_webview_window(&label).is_some() {
        return Ok(true);
    }

    let (width, height, url) = widget_config(&id).ok_or_else(|| format!("Unknown widget: {}", id))?;
    state.store.lock().unwrap().set(&format!("widget_{}_visible", id), json!(true));

    let sw = screen_width(&app);
    let sh = screen_height(&app);
    let x = (sw - width as i32) / 2 + rand_offset(&id);
    let y = (sh - height as i32) / 2 + rand_offset2(&id);

    WebviewWindowBuilder::new(&app, &label, WebviewUrl::App(url.into()))
        .title(id.clone())
        .inner_size(width as f64, height as f64)
        .position(x as f64, y as f64)
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .build()
        .map_err(|e| e.to_string())?;

    let _ = app.emit("widget-config-changed", json!({ "id": id, "visible": true }));
    Ok(true)
}

#[tauri::command]
pub async fn close_widget(id: String, app: tauri::AppHandle, state: State<'_, AppState>) -> Result<bool, String> {
    let label = format!("widget-{}", id);
    if let Some(win) = app.get_webview_window(&label) {
        win.close().map_err(|e| e.to_string())?;
    }
    state.store.lock().unwrap().set(&format!("widget_{}_visible", id), json!(false));
    let _ = app.emit("widget-config-changed", json!({ "id": id, "visible": false }));
    Ok(true)
}

#[tauri::command]
pub async fn get_widget_windows(app: tauri::AppHandle) -> Result<Vec<Value>, String> {
    let widgets = ["clock", "system-monitor", "weather", "notes", "music-player"];
    let mut result = Vec::new();
    for w in widgets {
        let label = format!("widget-{}", w);
        let visible = app.get_webview_window(&label).is_some();
        result.push(json!({ "id": w, "visible": visible }));
    }
    Ok(result)
}

fn screen_width(app: &tauri::AppHandle) -> i32 {
    app.primary_monitor()
        .ok()
        .flatten()
        .map(|m| (m.size().width as f64 / m.scale_factor()) as i32)
        .unwrap_or(1920)
}

fn screen_height(app: &tauri::AppHandle) -> i32 {
    app.primary_monitor()
        .ok()
        .flatten()
        .map(|m| (m.size().height as f64 / m.scale_factor()) as i32)
        .unwrap_or(1080)
}

fn rand_offset(id: &str) -> i32 {
    let hash: i32 = id.bytes().map(|b| b as i32).sum();
    (hash % 200) - 100
}

fn rand_offset2(id: &str) -> i32 {
    let hash: i32 = id.bytes().enumerate().map(|(i, b)| (i as i32) * (b as i32)).sum();
    (hash % 150) - 75
}

// ===== Launcher / Settings Pencereleri =====

#[tauri::command]
pub async fn open_launcher_window(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(win) = app.get_webview_window("launcher") {
        win.show().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
        return Ok(true);
    }
    let sw = screen_width(&app) as f64;
    let sh = screen_height(&app) as f64;
    WebviewWindowBuilder::new(&app, "launcher", WebviewUrl::App("launcher/index.html".into()))
        .title("Uygulama BaÅŸlatÄ±cÄ±")
        .inner_size(600.0, 450.0)
        .position((sw - 600.0) / 2.0, (sh - 450.0) / 2.0)
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub async fn close_launcher_window(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(win) = app.get_webview_window("launcher") {
        win.close().map_err(|e| e.to_string())?;
    }
    Ok(true)
}

#[tauri::command]
pub async fn toggle_launcher_window(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(win) = app.get_webview_window("launcher") {
        if win.is_visible().unwrap_or(false) {
            win.hide().map_err(|e| e.to_string())?;
        } else {
            win.show().map_err(|e| e.to_string())?;
            win.set_focus().map_err(|e| e.to_string())?;
        }
    } else {
        open_launcher_window(app).await?;
    }
    Ok(true)
}

#[tauri::command]
pub async fn open_settings_window(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(win) = app.get_webview_window("settings") {
        win.show().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
        return Ok(true);
    }
    let sw = screen_width(&app) as f64;
    let sh = screen_height(&app) as f64;
    WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App("settings/index.html".into()))
        .title("Ayarlar")
        .inner_size(900.0, 600.0)
        .position((sw - 900.0) / 2.0, (sh - 600.0) / 2.0)
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub async fn close_settings_window(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(win) = app.get_webview_window("settings") {
        win.close().map_err(|e| e.to_string())?;
    }
    Ok(true)
}

#[tauri::command]
pub async fn start_video_server(path: String) -> Result<u16, String> {
    crate::wallpaper::start_video_server(path)
}

#[tauri::command]
pub async fn set_window_size(app: tauri::AppHandle, label: String, width: f64, height: f64) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(&label) {
        win.set_size(tauri::PhysicalSize::new(width as u32, height as u32)).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub async fn toggle_settings_window(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(win) = app.get_webview_window("settings") {
        if win.is_visible().unwrap_or(false) {
            win.hide().map_err(|e| e.to_string())?;
        } else {
            win.show().map_err(|e| e.to_string())?;
            win.set_focus().map_err(|e| e.to_string())?;
        }
    } else {
        open_settings_window(app).await?;
    }
    Ok(true)
}

// ===== Pencere TaÅŸÄ±ma =====

#[tauri::command]
pub async fn move_window(app: tauri::AppHandle, label: String, x: i32, y: i32) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(&label) {
        win.set_position(tauri::PhysicalPosition::new(x, y)).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_window_position(app: tauri::AppHandle, label: String) -> Result<Value, String> {
    if let Some(win) = app.get_webview_window(&label) {
        let pos = win.outer_position().map_err(|e| e.to_string())?;
        Ok(json!({ "x": pos.x, "y": pos.y }))
    } else {
        Ok(json!({ "x": 0, "y": 0 }))
    }
}

#[tauri::command]
pub async fn set_taskbar_config(app: tauri::AppHandle, position: String, height: i32) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("taskbar") {
        let (mw, mh) = if let Ok(Some(monitor)) = win.primary_monitor() {
            (monitor.size().width as i32, monitor.size().height as i32)
        } else {
            (1920, 1080)
        };

        let (new_w, new_h, nx, ny) = match position.as_str() {
            "left" => {
                let w = 48i32;
                (w, mh, 0, 0)
            }
            "right" => {
                let w = 48i32;
                (w, mh, mw - w, 0)
            }
            "top" => {
                let w = 700i32.min(mw);
                (w, height, (mw - w) / 2, 0)
            }
            _ => {
                let w = 700i32.min(mw);
                (w, height, (mw - w) / 2, mh - height)
            }
        };

        win.set_size(tauri::PhysicalSize::new(new_w as u32, new_h as u32)).map_err(|e| e.to_string())?;
        win.set_position(tauri::PhysicalPosition::new(nx, ny)).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn create_wallpaper_window(app: tauri::AppHandle) -> Result<bool, String> {
    if app.get_webview_window("wallpaper").is_some() {
        return Ok(true);
    }

    let (sw, sh) = {
        let monitor = app.primary_monitor().ok().flatten();
        match monitor {
            Some(m) => (m.size().width as f64, m.size().height as f64),
            None => (1920.0, 1080.0),
        }
    };

    let win = WebviewWindowBuilder::new(&app, "wallpaper", WebviewUrl::App("wallpaper/index.html".into()))
        .title("detbar Wallpaper")
        .inner_size(sw, sh)
        .position(0.0, 0.0)
        .decorations(false)
        .resizable(false)
        .skip_taskbar(true)
        .build()
        .map_err(|e| e.to_string())?;

    // Wallpaper'i masaustu ikonlarinin ARKASINA yerlestir (WorkerW embed) - retry ile
    #[cfg(target_os = "windows")]
    {
        // HWND'yi sync olarak al, sonra async'a ver
        let maybe_hwnd = win.hwnd().ok().map(|h| h.0 as isize);
        if let Some(hwnd) = maybe_hwnd {
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let mut embedded = false;
                for i in 0..3 {
                    match crate::wallpaper::embed_behind_desktop(hwnd) {
                        Ok(()) => { embedded = true; break; }
                        Err(e) => {
                            eprintln!("[wallpaper] Embed deneme {}/3 basarisiz: {}", i + 1, e);
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        }
                    }
                }
                if !embedded {
                    eprintln!("[wallpaper] Embed tamamen basarisiz! Pencere uste kalacak.");
                }
                crate::wallpaper::make_noactivate(hwnd);
                crate::wallpaper::disable_context_menu(hwnd);
            });
        }
    }

    // Taskbar ve Desktop'i one getir
    if let Some(desk) = app.get_webview_window("desktop") {
        let _ = desk.set_focus();
    }
    if let Some(tb) = app.get_webview_window("taskbar") {
        let _ = tb.set_focus();
    }

    Ok(true)
}

#[tauri::command]
pub async fn stop_video_server() -> Result<(), String> {
    crate::wallpaper::stop_video_server();
    Ok(())
}

// ===== Sanal MasaÃ¼stÃ¼ =====

#[tauri::command]
pub async fn get_desktops(state: State<'_, AppState>) -> Result<Value, String> {
    Ok(state.desktop_manager.lock().unwrap().to_json())
}

#[tauri::command]
pub async fn switch_desktop(state: State<'_, AppState>, id: usize) -> Result<(), String> {
    let all = crate::virtual_desktop::enum_windows();
    let hwnds: Vec<isize> = all.iter().map(|w| w.hwnd).collect();
    let taskbar_h = state.store.lock().unwrap().get("taskbarHeight").as_i64().unwrap_or(48) as i32;
    state.desktop_manager.lock().unwrap().switch_to(id, &hwnds, taskbar_h);
    Ok(())
}

#[tauri::command]
pub async fn create_desktop(state: State<'_, AppState>, name: String) -> Result<(), String> {
    let mut mgr = state.desktop_manager.lock().unwrap();
    let id = mgr.desktops.len();
    mgr.desktops.push(crate::virtual_desktop::Desktop {
        id,
        name,
        shortcuts: vec![],
        widget_ids: vec![],
        window_hwnds: vec![],
    });
    drop(mgr);
    state.desktop_manager.lock().unwrap().save();
    Ok(())
}

#[tauri::command]
pub async fn delete_desktop(state: State<'_, AppState>, id: usize) -> Result<(), String> {
    let mut mgr = state.desktop_manager.lock().unwrap();
    if mgr.desktops.len() <= 1 { return Err("Son masaüstü silinemez".into()); }
    if id >= mgr.desktops.len() { return Err("Geçersiz masaüstü".into()); }
    mgr.desktops.remove(id);
    for (i, d) in mgr.desktops.iter_mut().enumerate() { d.id = i; }
    if mgr.current >= mgr.desktops.len() { mgr.current = mgr.desktops.len() - 1; }
    else if mgr.current > id { mgr.current -= 1; }
    drop(mgr);
    state.desktop_manager.lock().unwrap().save();
    Ok(())
}

#[tauri::command]
pub async fn rename_desktop(state: State<'_, AppState>, id: usize, name: String) -> Result<(), String> {
    let mut mgr = state.desktop_manager.lock().unwrap();
    if id < mgr.desktops.len() {
        mgr.desktops[id].name = name;
        drop(mgr);
        state.desktop_manager.lock().unwrap().save();
    }
    Ok(())
}

#[tauri::command]
pub async fn enum_open_windows() -> Result<Value, String> {
    let wins = crate::virtual_desktop::enum_windows();
    Ok(json!(wins.iter().map(|w| json!({
        "hwnd": w.hwnd,
        "title": w.title,
        "class": w.class,
        "pid": w.pid,
    })).collect::<Vec<_>>()))
}

#[tauri::command]
pub async fn move_app_window(hwnd: isize, x: i32, y: i32, w: i32, h: i32) -> Result<(), String> {
    crate::virtual_desktop::set_window_bounds(hwnd, x, y, w, h);
    Ok(())
}

#[tauri::command]
pub async fn show_app_window(hwnd: isize) -> Result<(), String> {
    crate::virtual_desktop::show_window(hwnd);
    Ok(())
}

#[tauri::command]
pub async fn hide_app_window(hwnd: isize) -> Result<(), String> {
    crate::virtual_desktop::hide_window(hwnd);
    Ok(())
}

#[tauri::command]
pub async fn add_desktop_shortcut(state: State<'_, AppState>, desktop_id: usize, name: String, path: String, icon: String) -> Result<(), String> {
    let mut mgr = state.desktop_manager.lock().unwrap();
    if desktop_id < mgr.desktops.len() {
        let count = mgr.desktops[desktop_id].shortcuts.len();
        let cols = 6;
        let col = (count % cols) as f64;
        let row = (count / cols) as f64;
        let x = 40.0 + col * 90.0;
        let y = 40.0 + row * 100.0;
        mgr.desktops[desktop_id].shortcuts.push(crate::virtual_desktop::Shortcut {
            name, path, icon, x, y,
        });
        drop(mgr);
        state.desktop_manager.lock().unwrap().save();
    }
    Ok(())
}

#[tauri::command]
pub async fn update_shortcut_position(state: State<'_, AppState>, desktop_id: usize, index: usize, x: f64, y: f64) -> Result<(), String> {
    let mut mgr = state.desktop_manager.lock().unwrap();
    if desktop_id < mgr.desktops.len() && index < mgr.desktops[desktop_id].shortcuts.len() {
        mgr.desktops[desktop_id].shortcuts[index].x = x;
        mgr.desktops[desktop_id].shortcuts[index].y = y;
        drop(mgr);
        state.desktop_manager.lock().unwrap().save();
    }
    Ok(())
}

#[tauri::command]
pub async fn delete_desktop_shortcut(state: State<'_, AppState>, desktop_id: usize, index: usize) -> Result<(), String> {
    let mut mgr = state.desktop_manager.lock().unwrap();
    if desktop_id < mgr.desktops.len() && index < mgr.desktops[desktop_id].shortcuts.len() {
        mgr.desktops[desktop_id].shortcuts.remove(index);
        drop(mgr);
        state.desktop_manager.lock().unwrap().save();
    }
    Ok(())
}
#[tauri::command]
pub async fn show_desktop(app: tauri::AppHandle) -> Result<bool, String> {
    let (sw, sh) = {
        let monitor = app.primary_monitor().ok().flatten();
        match monitor {
            Some(m) => (m.size().width as f64, m.size().height as f64),
            None => (1920.0, 1080.0),
        }
    };

    if let Some(win) = app.get_webview_window("desktop") {
        win.set_size(tauri::PhysicalSize::new(sw as u32, sh as u32)).map_err(|e| e.to_string())?;
        win.set_position(tauri::PhysicalPosition::new(0, 0)).map_err(|e| e.to_string())?;
        win.show().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
        return Ok(true);
    }

    let win = WebviewWindowBuilder::new(&app, "desktop", WebviewUrl::App("desktop/index.html".into()))
        .title("detbar Desktop")
        .inner_size(sw, sh)
        .position(0.0, 0.0)
        .decorations(false)
        .resizable(false)
        .skip_taskbar(true)
        .build()
        .map_err(|e| e.to_string())?;

    win.show().map_err(|e| e.to_string())?;

    // WebView2 context menu'yu native seviyede kapat
    #[cfg(target_os = "windows")]
    {
        let hwnd_val = win.hwnd().map_err(|_| "hwnd error".to_string())?;
        let raw: isize = unsafe { std::mem::transmute(hwnd_val) };
        crate::wallpaper::disable_context_menu(raw);
    }

    // Taskbar'i one getir
    if let Some(tb) = app.get_webview_window("taskbar") {
        let _ = tb.set_focus();
    }

    Ok(true)
}

#[tauri::command]
pub async fn tile_current_desktop(state: State<'_, AppState>) -> Result<(), String> {
    let taskbar_h = state.store.lock().unwrap().get("taskbarHeight").as_i64().unwrap_or(48) as i32;
    
    let wins = crate::virtual_desktop::enum_windows();
    
    for w in &wins {
        if w.title.contains("detbar Desktop") || w.title.contains("detbar Wallpaper") {
            crate::virtual_desktop::send_to_bottom(w.hwnd);
        }
    }
    
    let mgr = state.desktop_manager.lock().unwrap();
    let cur = mgr.current;
    let to_tile: Vec<isize> = mgr.desktops[cur].window_hwnds.iter()
        .filter(|hwnd| wins.iter().any(|w| w.hwnd == **hwnd))
        .cloned()
        .collect();
    drop(mgr);
    
    for hwnd in &to_tile {
        crate::virtual_desktop::bring_to_top(*hwnd);
    }
    crate::virtual_desktop::tile_desktop_windows(&to_tile, taskbar_h);
    
    Ok(())
}

#[tauri::command]
pub async fn get_auto_assign_windows(state: State<'_, AppState>) -> Result<bool, String> {
    let store = state.store.lock().unwrap();
    Ok(store.get("autoAssignWindows").as_bool().unwrap_or(true))
}

#[tauri::command]
pub async fn set_auto_assign_windows(state: State<'_, AppState>, value: bool) -> Result<(), String> {
    let mut store = state.store.lock().unwrap();
    store.set("autoAssignWindows", json!(value));
    Ok(())
}

#[tauri::command]
pub async fn get_app_desktop_map(state: State<'_, AppState>) -> Result<Value, String> {
    let store = state.store.lock().unwrap();
    Ok(store.get("appDesktopMap"))
}

#[tauri::command]
pub async fn set_app_desktop_map(state: State<'_, AppState>, map: Value) -> Result<(), String> {
    let mut store = state.store.lock().unwrap();
    store.set("appDesktopMap", map);
    Ok(())
}

#[tauri::command]
pub async fn assign_window_to_desktop(state: State<'_, AppState>, hwnd: isize, desktop_id: usize) -> Result<(), String> {
    let mut mgr = state.desktop_manager.lock().unwrap();
    for d in &mut mgr.desktops { d.window_hwnds.retain(|h| *h != hwnd); }
    if desktop_id < mgr.desktops.len() { mgr.desktops[desktop_id].window_hwnds.push(hwnd); }
    drop(mgr);
    if let Some(proc_name) = crate::virtual_desktop::get_process_name(hwnd) {
        let mut store = state.store.lock().unwrap();
        let mut map = store.get("appDesktopMap").as_object().cloned().unwrap_or_default();
        map.insert(proc_name, json!(desktop_id));
        store.set("appDesktopMap", json!(map));
        store.set("autoAssignWindows", json!(true));
    }
    Ok(())
}

#[tauri::command]
pub async fn promote_master(state: State<'_, AppState>, hwnd: isize) -> Result<(), String> {
    let mut mgr = state.desktop_manager.lock().unwrap();
    let cur = mgr.current;
    crate::virtual_desktop::promote_to_master(&mut mgr.desktops[cur].window_hwnds, hwnd);
    Ok(())
}

#[tauri::command]
pub async fn move_window_to_desktop(state: State<'_, AppState>, hwnd: isize, to_id: usize, remember: Option<bool>) -> Result<(), String> {
    let mut mgr = state.desktop_manager.lock().unwrap();
    // Remove from all desktops
    for d in &mut mgr.desktops {
        d.window_hwnds.retain(|h| *h != hwnd);
    }
    // Add to target desktop
    if to_id < mgr.desktops.len() {
        mgr.desktops[to_id].window_hwnds.push(hwnd);
    }
    drop(mgr);
    
    // Remember mapping for this app
    if remember.unwrap_or(false) {
        if let Some(proc_name) = crate::virtual_desktop::get_process_name(hwnd) {
            let mut store = state.store.lock().unwrap();
            let mut map = store.get("appDesktopMap").as_object().cloned().unwrap_or_default();
            map.insert(proc_name, json!(to_id));
            store.set("appDesktopMap", json!(map));
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn get_window_under_cursor() -> Result<Value, String> {
    match crate::virtual_desktop::get_window_under_cursor() {
        Some(w) => Ok(json!({
            "hwnd": w.hwnd,
            "title": w.title,
            "class": w.class,
            "pid": w.pid,
        })),
        None => Ok(json!(null)),
    }
}

#[tauri::command]
pub async fn get_current_mouse_pos() -> Result<Value, String> {
    extern "system" {
        fn GetCursorPos(lpPoint: *mut MousePoint) -> i32;
    }
    #[repr(C)]
    struct MousePoint { x: i32, y: i32 }
    let mut pt = MousePoint { x: 0, y: 0 };
    unsafe { GetCursorPos(&mut pt); }
    Ok(json!({ "x": pt.x, "y": pt.y }))
}

#[tauri::command]
pub async fn remove_window_from_all_desktops(state: State<'_, AppState>, hwnd: isize) -> Result<(), String> {
    let mut mgr = state.desktop_manager.lock().unwrap();
    for d in &mut mgr.desktops {
        d.window_hwnds.retain(|h| *h != hwnd);
    }
    Ok(())
}