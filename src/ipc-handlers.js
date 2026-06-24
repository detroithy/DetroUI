const path = require('path');
const { exec } = require('child_process');
const { promisify } = require('util');

const execAsync = promisify(exec);

function setupIpcHandlers(ipcMain, store, managers) {
  const si = require('systeminformation');
  const { BrowserWindow } = require('electron');

  // Tüm pencerelere tema değişikliğini bildir
  function broadcastThemeChange(theme, accentColor) {
    BrowserWindow.getAllWindows().forEach(win => {
      if (!win.isDestroyed()) {
        win.webContents.send('theme-changed', { theme, accentColor });
      }
    });
  }

  // Tüm pencerelere widget değişikliğini bildir
  function broadcastWidgetChange() {
    BrowserWindow.getAllWindows().forEach(win => {
      if (!win.isDestroyed()) {
        win.webContents.send('widget-config-changed');
      }
    });
  }

  // Sistem bilgileri
  ipcMain.handle('get-cpu-load', async () => {
    try {
      const load = await si.currentLoad();
      return {
        currentLoad: load.currentLoad,
        cpus: load.cpus.map(cpu => ({
          load: cpu.load,
          user: cpu.loadUser,
          system: cpu.loadSystem
        }))
      };
    } catch (error) {
      return { currentLoad: 0, cpus: [] };
    }
  });

  ipcMain.handle('get-memory', async () => {
    try {
      const mem = await si.mem();
      return {
        total: mem.total,
        free: mem.free,
        used: mem.used,
        usedPercent: ((mem.used / mem.total) * 100).toFixed(1)
      };
    } catch (error) {
      return { total: 0, free: 0, used: 0, usedPercent: 0 };
    }
  });

  ipcMain.handle('get-disk', async () => {
    try {
      const disks = await si.fsSize();
      return disks.map(disk => ({
        fs: disk.fs,
        type: disk.type,
        size: disk.size,
        used: disk.used,
        available: disk.available,
        usedPercent: disk.use
      }));
    } catch (error) {
      return [];
    }
  });

  ipcMain.handle('get-battery', async () => {
    try {
      const battery = await si.battery();
      return {
        level: battery.level,
        charging: battery.charging,
        timeRemaining: battery.timeRemaining,
        type: battery.type
      };
    } catch (error) {
      return { level: 100, charging: false, timeRemaining: null };
    }
  });

  ipcMain.handle('get-cpu-temp', async () => {
    try {
      const temp = await si.cpuTemperature();
      return {
        main: temp.main,
        max: temp.max,
        cores: temp.cores
      };
    } catch (error) {
      return { main: 0, max: 0, cores: [] };
    }
  });

  ipcMain.handle('get-network', async () => {
    try {
      const stats = await si.networkStats();
      const interfaces = await si.networkInterfaces();
      return {
        stats: stats.map(s => ({
          iface: s.iface,
          rxBytes: s.rx_bytes,
          txBytes: s.tx_bytes,
          rxSec: s.rx_sec,
          txSec: s.tx_sec
        })),
        interfaces: interfaces.map(i => ({
          iface: i.iface,
          ip4: i.ip4,
          mac: i.mac,
          speed: i.speed,
          type: i.type
        }))
      };
    } catch (error) {
      return { stats: [], interfaces: [] };
    }
  });

  // Hava durumu
  ipcMain.handle('get-weather', async (event, city) => {
    try {
      const { fetchWeatherApi } = require('openmeteo');

      const coords = {
        'istanbul': { lat: 41.0082, lon: 28.9784 },
        'ankara': { lat: 39.9334, lon: 32.8597 },
        'izmir': { lat: 38.4192, lon: 27.1287 },
        'london': { lat: 51.5074, lon: -0.1278 },
        'new york': { lat: 40.7128, lon: -74.006 },
        'berlin': { lat: 52.52, lon: 13.405 }
      };

      const coord = coords[city.toLowerCase()] || coords['istanbul'];

      const params = {
        latitude: coord.lat,
        longitude: coord.lon,
        current: 'temperature_2m,relative_humidity_2m,weather_code,wind_speed_10m,apparent_temperature',
        daily: 'weather_code,temperature_2m_max,temperature_2m_min',
        timezone: 'auto',
        forecast_days: 5
      };

      const url = 'https://api.open-meteo.com/v1/forecast';
      const responses = await fetchWeatherApi(url, params);
      const response = responses[0];

      const current = response.current();
      const daily = response.daily();

      const weatherCodes = {
        0: { desc: 'Güneşli', icon: '☀️' },
        1: { desc: 'Az Bulutlu', icon: '🌤️' },
        2: { desc: 'Parçalı Bulutlu', icon: '⛅' },
        3: { desc: 'Kapalı', icon: '☁️' },
        45: { desc: 'Sisli', icon: '🌫️' },
        48: { desc: 'Kırağılı Sis', icon: '🌫️' },
        51: { desc: 'Hafif Çiseleme', icon: '🌦️' },
        53: { desc: 'Orta Çiseleme', icon: '🌦️' },
        55: { desc: 'Yoğun Çiseleme', icon: '🌧️' },
        61: { desc: 'Hafif Yağmur', icon: '🌧️' },
        63: { desc: 'Orta Yağmur', icon: '🌧️' },
        65: { desc: 'Şiddetli Yağmur', icon: '🌧️' },
        71: { desc: 'Hafif Kar', icon: '❄️' },
        73: { desc: 'Orta Kar', icon: '❄️' },
        75: { desc: 'Yoğun Kar', icon: '❄️' },
        95: { desc: 'Fırtına', icon: '⛈️' },
        96: { desc: 'Hafif Dolu Fırtına', icon: '⛈️' },
        99: { desc: 'Şiddetli Dolu Fırtına', icon: '⛈️' }
      };

      const weatherCode = current.variables(2).value();
      const weatherInfo = weatherCodes[weatherCode] || { desc: 'Bilinmiyor', icon: '❓' };

      return {
        current: {
          temperature: current.variables(0).value(),
          humidity: current.variables(1).value(),
          weatherCode: weatherCode,
          description: weatherInfo.desc,
          icon: weatherInfo.icon,
          windSpeed: current.variables(4).value(),
          feelsLike: current.variables(3).value()
        },
        daily: Array.from({ length: 5 }, (_, i) => ({
          date: new Date(daily.time[i]).toLocaleDateString('tr-TR', { weekday: 'short', day: 'numeric', month: 'short' }),
          max: daily.variables(1).valuesArray()[i],
          min: daily.variables(2).valuesArray()[i],
          weatherCode: daily.variables(0).valuesArray()[i],
          description: (weatherCodes[daily.variables(0).valuesArray()[i]] || weatherCodes[0]).desc,
          icon: (weatherCodes[daily.variables(0).valuesArray()[i]] || weatherCodes[0]).icon
        })),
        city: city
      };
    } catch (error) {
      console.error('Weather error:', error);
      return {
        current: { temperature: 0, humidity: 0, description: 'Hata', icon: '❌', windSpeed: 0, feelsLike: 0 },
        daily: [],
        city: city
      };
    }
  });

  // Uygulamalar
  ipcMain.handle('get-installed-apps', async () => {
    try {
      const { stdout } = await execAsync(`
        $apps = @()
        $paths = @(
          'HKLM:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\*',
          'HKLM:\\Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\*',
          'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\*'
        )
        foreach ($path in $paths) {
          if (Test-Path $path) {
            Get-ItemProperty $path | Where-Object {$_.DisplayName} | ForEach-Object {
              $apps += @{
                name = $_.DisplayName
                version = $_.DisplayVersion
                publisher = $_.Publisher
                path = $_.InstallLocation
              }
            }
          }
        }
        $apps | ConvertTo-Json -Compress
      `, { shell: 'powershell', encoding: 'utf8' });
      
      let apps = JSON.parse(stdout || '[]');
      if (!Array.isArray(apps)) apps = [apps];
      return apps.filter(app => app.name);
    } catch (error) {
      return [];
    }
  });

  ipcMain.on('launch-app', (event, appPath) => {
    if (appPath && typeof appPath === 'string') {
      exec(`start "" "${appPath}"`, (err) => {
        if (err) console.error('Launch error:', err);
      });
    }
  });

  // Müzik kontrolü
  ipcMain.handle('get-media-sessions', async () => {
    try {
      const { stdout } = await execAsync(`
        Add-Type @"
          using System;
          using System.Runtime.InteropServices;
          public class WinIcon {
            [DllImport("shell32.dll")] public static extern IntPtr ExtractIcon(IntPtr hInst, string file, int index);
          }
"@
        $sessions = @()
        $musicApps = @('Spotify','vlc','mpc-hc64','foobar2000','MusicBee','AIMP','Winamp','iTunes','Groove','Media','PotPlayer')
        Get-Process | Where-Object {
          $_.MainWindowTitle -ne "" -and (
            $_.ProcessName -match 'spotify|vlc|music|player|media|groove|aimp|foobar|winamp|itunes|potplayer|mpc' -or
            $_.MainWindowTitle -match ' - |–|—'
          )
        } | ForEach-Object {
          $title = $_.MainWindowTitle
          $artist = ""
          $songTitle = $title
          if ($title -match '^(.+?)\\s*[–—-]\\s+(.+)$') {
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
      `, { shell: 'powershell', encoding: 'utf8', maxBuffer: 1024*1024 });

      let sessions = JSON.parse(stdout || '[]');
      if (!Array.isArray(sessions)) sessions = [sessions];
      return sessions.filter(s => s.appName);
    } catch (error) {
      return [];
    }
  });

  ipcMain.on('media-play', (event, appName) => {
    try {
      execAsync('$obj = New-Object -ComObject WScript.Shell; $obj.SendKeys([char]179)', { shell: 'powershell' });
    } catch (error) {}
  });

  ipcMain.on('media-pause', (event, appName) => {
    try {
      execAsync('$obj = New-Object -ComObject WScript.Shell; $obj.SendKeys([char]179)', { shell: 'powershell' });
    } catch (error) {}
  });

  ipcMain.on('media-next', (event, appName) => {
    try {
      execAsync('$obj = New-Object -ComObject WScript.Shell; $obj.SendKeys([char]176)', { shell: 'powershell' });
    } catch (error) {}
  });

  ipcMain.on('media-prev', (event, appName) => {
    try {
      execAsync('$obj = New-Object -ComObject WScript.Shell; $obj.SendKeys([char]177)', { shell: 'powershell' });
    } catch (error) {}
  });

  ipcMain.on('media-toggle', () => {
    try {
      execAsync('$obj = New-Object -ComObject WScript.Shell; $obj.SendKeys([char]179)', { shell: 'powershell' });
    } catch (error) {}
  });

  ipcMain.on('media-seek', (event, percent) => {
    try {
      const cmd = '$obj = New-Object -ComObject WScript.Shell; $obj.SendKeys("^({RIGHT})")';
      execAsync(cmd, { shell: 'powershell' });
    } catch (error) {}
  });

  // Pencere kontrolü
  ipcMain.on('window-minimize', (event, id) => {
    // Pencereyi küçült
  });

  ipcMain.on('window-close', (event, id) => {
    // Pencereyi kapat
  });

  ipcMain.on('window-focus', (event, id) => {
    // Pencereyi öne getir
  });

  ipcMain.handle('get-open-windows', () => {
    return [];
  });

  // Widget yönetimi
  ipcMain.on('create-widget', (event, widgetId) => {
    managers.createWidget(widgetId);
    broadcastWidgetChange();
  });

  ipcMain.on('close-widget', (event, widgetId) => {
    const win = managers.widgetWindows.get(widgetId);
    if (win && !win.isDestroyed()) {
      win.close();
      broadcastWidgetChange();
    }
  });

  ipcMain.on('show-all-widgets', () => {
    managers.showAllWidgets();
  });

  ipcMain.on('hide-all-widgets', () => {
    managers.hideAllWidgets();
  });

  ipcMain.handle('get-widget-windows', () => {
    const windows = [];
    managers.widgetWindows.forEach((win, id) => {
      if (!win.isDestroyed()) {
        windows.push({ id, visible: win.isVisible() });
      }
    });
    return windows;
  });

  // Launcher
  ipcMain.on('open-launcher', () => {
    const launcher = managers.launcherWindow();
    if (launcher && !launcher.isDestroyed()) {
      launcher.show();
    }
  });

  ipcMain.on('close-launcher', () => {
    const launcher = managers.launcherWindow();
    if (launcher && !launcher.isDestroyed()) {
      launcher.hide();
    }
  });

  ipcMain.on('toggle-launcher', () => {
    const launcher = managers.launcherWindow();
    if (launcher && !launcher.isDestroyed()) {
      if (launcher.isVisible()) launcher.hide();
      else launcher.show();
    }
  });

  // Settings
  ipcMain.on('open-settings', () => {
    managers.createSettings();
    const settings = managers.settingsWindow();
    if (settings && !settings.isDestroyed()) {
      settings.show();
    }
  });

  ipcMain.on('close-settings', () => {
    const settings = managers.settingsWindow();
    if (settings && !settings.isDestroyed()) {
      settings.hide();
    }
  });

  // Ayarlar - Store
  ipcMain.handle('get-store-value', (event, key) => {
    return store.get(key);
  });

  ipcMain.handle('set-store-value', (event, key, value) => {
    store.set(key, value);
    if (key === 'taskbarPosition' || key === 'taskbarHeight') {
      const pos = store.get('taskbarPosition');
      const h = store.get('taskbarHeight');
      if (managers.repositionTaskbar) managers.repositionTaskbar(pos, h);
      BrowserWindow.getAllWindows().forEach(win => {
        if (!win.isDestroyed()) {
          win.webContents.send('taskbar-settings-changed', { position: pos, height: h });
        }
      });
    }
    return true;
  });

  // Tema
  ipcMain.handle('get-theme', () => {
    return store.get('theme');
  });

  ipcMain.handle('set-theme', (event, theme) => {
    store.set('theme', theme);
    broadcastThemeChange(theme, store.get('accentColor'));
    return true;
  });

  ipcMain.handle('get-accent-color', () => {
    return store.get('accentColor');
  });

  ipcMain.handle('set-accent-color', (event, color) => {
    store.set('accentColor', color);
    broadcastThemeChange(store.get('theme'), color);
    return true;
  });

  // Notlar
  ipcMain.handle('get-notes', () => {
    return store.get('notes') || [];
  });

  ipcMain.handle('save-notes', (event, notes) => {
    store.set('notes', notes);
    return true;
  });

  // Widget pencere taşıma
  ipcMain.on('window-move', (event, x, y) => {
    const win = BrowserWindow.fromWebContents(event.sender);
    if (win && !win.isDestroyed()) {
      win.setPosition(Math.round(x), Math.round(y));
    }
  });

  ipcMain.handle('get-window-position', (event) => {
    const win = BrowserWindow.fromWebContents(event.sender);
    if (win && !win.isDestroyed()) {
      const [x, y] = win.getPosition();
      return { x, y };
    }
    return { x: 0, y: 0 };
  });

  ipcMain.handle('set-window-position', (event, x, y) => {
    const win = BrowserWindow.fromWebContents(event.sender);
    if (win && !win.isDestroyed()) {
      win.setPosition(Math.round(x), Math.round(y));
      return true;
    }
    return false;
  });

  ipcMain.handle('screen-information', () => {
    const { screen } = require('electron');
    const display = screen.getPrimaryDisplay();
    return {
      width: display.bounds.width,
      height: display.bounds.height,
      workArea: display.workArea,
      scaleFactor: display.scaleFactor
    };
  });

  // Görev çubuğu ayarları
  ipcMain.handle('get-taskbar-settings', () => {
    return {
      position: store.get('taskbarPosition'),
      height: store.get('taskbarHeight')
    };
  });

  ipcMain.handle('set-taskbar-settings', (event, settings) => {
    if (settings.position) store.set('taskbarPosition', settings.position);
    if (settings.height) store.set('taskbarHeight', settings.height);
    const pos = settings.position || store.get('taskbarPosition');
    const h = settings.height || store.get('taskbarHeight');
    if (managers.repositionTaskbar) managers.repositionTaskbar(pos, h);
    BrowserWindow.getAllWindows().forEach(win => {
      if (!win.isDestroyed()) {
        win.webContents.send('taskbar-settings-changed', { position: pos, height: h });
      }
    });
    return true;
  });

  // Opaklik - tum pencerelere uygula
  ipcMain.on('set-opacity', (event, value) => {
    store.set('opacity', value);
    if (managers.setOpacity) managers.setOpacity(value);
  });

  // Wallpaper
  ipcMain.handle('pick-wallpaper-file', async () => {
    const { dialog } = require('electron');
    const result = await dialog.showOpenDialog({
      title: 'Animasyonlu Wallpaper Seç',
      filters: [
        { name: 'Video Dosyaları', extensions: ['mp4', 'webm', 'avi', 'mkv', 'mov'] },
        { name: 'Tüm Dosyalar', extensions: ['*'] }
      ],
      properties: ['openFile']
    });
    if (result.canceled || !result.filePaths.length) return null;
    return result.filePaths[0];
  });

  ipcMain.on('set-wallpaper', (event, filePath) => {
    store.set('wallpaperPath', filePath);
    if (!managers.wallpaperWindow || managers.wallpaperWindow().isDestroyed()) {
      if (managers.createWallpaper) managers.createWallpaper();
    }
    const wp = managers.wallpaperWindow();
    if (wp && !wp.isDestroyed()) {
      wp.webContents.send('load-wallpaper', { path: filePath, volume: store.get('wallpaperVolume') || 0 });
      wp.show();
    }
  });

  ipcMain.on('stop-wallpaper', () => {
    store.set('wallpaperPath', '');
    const wp = managers.wallpaperWindow();
    if (wp && !wp.isDestroyed()) {
      wp.webContents.send('wallpaper-stop');
    }
  });

  ipcMain.on('wallpaper-volume', (event, vol) => {
    store.set('wallpaperVolume', vol);
    const wp = managers.wallpaperWindow();
    if (wp && !wp.isDestroyed()) {
      wp.webContents.send('wallpaper-volume-set', vol);
    }
  });

  ipcMain.on('wallpaper-play-pause', (event, play) => {
    const wp = managers.wallpaperWindow();
    if (wp && !wp.isDestroyed()) {
      wp.webContents.send('wallpaper-play-pause-set', play);
    }
  });
}

module.exports = { setupIpcHandlers };
