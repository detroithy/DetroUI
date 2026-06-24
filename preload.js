const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('plasmaAPI', {
  // Sistem bilgileri
  getCpuLoad: () => ipcRenderer.invoke('get-cpu-load'),
  getMemory: () => ipcRenderer.invoke('get-memory'),
  getDisk: () => ipcRenderer.invoke('get-disk'),
  getBattery: () => ipcRenderer.invoke('get-battery'),
  getCpuTemp: () => ipcRenderer.invoke('get-cpu-temp'),
  getNetwork: () => ipcRenderer.invoke('get-network'),

  // Hava durumu
  getWeather: (city) => ipcRenderer.invoke('get-weather', city),

  // Uygulamalar
  getInstalledApps: () => ipcRenderer.invoke('get-installed-apps'),
  launchApp: (path) => ipcRenderer.send('launch-app', path),

  // Müzik kontrolü
  getMediaSessions: () => ipcRenderer.invoke('get-media-sessions'),
  mediaPlay: (app) => ipcRenderer.send('media-play', app),
  mediaPause: (app) => ipcRenderer.send('media-pause', app),
  mediaNext: (app) => ipcRenderer.send('media-next', app),
  mediaPrev: (app) => ipcRenderer.send('media-prev', app),
  mediaToggle: () => ipcRenderer.send('media-toggle'),
  mediaSeek: (percent) => ipcRenderer.send('media-seek', percent),

  // Pencere kontrolü
  minimizeWindow: (id) => ipcRenderer.send('window-minimize', id),
  maximizeWindow: (id) => ipcRenderer.send('window-maximize', id),
  closeWindow: (id) => ipcRenderer.send('window-close', id),
  focusWindow: (id) => ipcRenderer.send('window-focus', id),
  getOpenWindows: () => ipcRenderer.invoke('get-open-windows'),

  // Widget yönetimi
  createWidget: (id) => ipcRenderer.send('create-widget', id),
  closeWidget: (id) => ipcRenderer.send('close-widget', id),
  showAllWidgets: () => ipcRenderer.send('show-all-widgets'),
  hideAllWidgets: () => ipcRenderer.send('hide-all-widgets'),
  getWidgetWindows: () => ipcRenderer.invoke('get-widget-windows'),

  // Launcher
  openLauncher: () => ipcRenderer.send('open-launcher'),
  closeLauncher: () => ipcRenderer.send('close-launcher'),
  toggleLauncher: () => ipcRenderer.send('toggle-launcher'),

  // Settings
  openSettings: () => ipcRenderer.send('open-settings'),
  closeSettings: () => ipcRenderer.send('close-settings'),

  // Ayarlar
  getStoreValue: (key) => ipcRenderer.invoke('get-store-value', key),
  setStoreValue: (key, value) => ipcRenderer.invoke('set-store-value', key, value),

  // Pencere sürükleme (widget konumlandırma)
  moveWindow: (x, y) => ipcRenderer.send('window-move', x, y),
  getWindowPosition: () => ipcRenderer.invoke('get-window-position'),
  setWindowPosition: (x, y) => ipcRenderer.invoke('set-window-position', x, y),
  startDrag: (windowType, id) => ipcRenderer.send('start-drag', windowType, id),
  screenInformation: () => ipcRenderer.invoke('screen-information'),

  // Tema
  getTheme: () => ipcRenderer.invoke('get-theme'),
  setTheme: (theme) => ipcRenderer.invoke('set-theme', theme),
  getAccentColor: () => ipcRenderer.invoke('get-accent-color'),
  setAccentColor: (color) => ipcRenderer.invoke('set-accent-color', color),
  setOpacity: (value) => ipcRenderer.send('set-opacity', value),

  // Notlar
  getNotes: () => ipcRenderer.invoke('get-notes'),
  saveNotes: (notes) => ipcRenderer.invoke('save-notes', notes),

  // Kısayollar
  registerShortcut: (key, action) => ipcRenderer.send('register-shortcut', key, action),

  // Event listeners
  onSystemUpdate: (callback) => ipcRenderer.on('system-update', (event, data) => callback(data)),
  onWidgetConfigChanged: (callback) => ipcRenderer.on('widget-config-changed', (event, data) => callback(data)),
  onThemeChanged: (callback) => ipcRenderer.on('theme-changed', (event, data) => callback(data)),
  onTaskbarSettingsChanged: (callback) => ipcRenderer.on('taskbar-settings-changed', (event, data) => callback(data)),

  // Wallpaper
  setWallpaper: (filePath) => ipcRenderer.send('set-wallpaper', filePath),
  stopWallpaper: () => ipcRenderer.send('stop-wallpaper'),
  setWallpaperVolume: (vol) => ipcRenderer.send('wallpaper-volume', vol),
  setWallpaperPlayPause: (play) => ipcRenderer.send('wallpaper-play-pause', play),
  pickWallpaperFile: () => ipcRenderer.invoke('pick-wallpaper-file'),
  onLoadWallpaper: (callback) => ipcRenderer.on('load-wallpaper', (event, data) => callback(data)),
  onWallpaperVolume: (callback) => ipcRenderer.on('wallpaper-volume-set', (event, vol) => callback(vol)),
  onWallpaperPlayPause: (callback) => ipcRenderer.on('wallpaper-play-pause-set', (event, play) => callback(play)),
  onWallpaperStop: (callback) => ipcRenderer.on('wallpaper-stop', () => callback())
});
