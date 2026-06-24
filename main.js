const { app, BrowserWindow, screen, Tray, Menu, ipcMain, globalShortcut, nativeImage } = require('electron');
const path = require('path');
const Store = require('electron-store');
const { setupIpcHandlers } = require('./src/ipc-handlers');

const store = new Store({
  defaults: {
    theme: 'dark',
    accentColor: '#00d4ff',
    taskbarPosition: 'bottom',
    taskbarHeight: 48,
    widgetPositions: {},
    enabledWidgets: ['clock', 'system-monitor'],
    launcherPinnedApps: [],
    weatherCity: 'Istanbul'
  }
});

let taskbarWindow = null;
let launcherWindow = null;
let settingsWindow = null;
let wallpaperWindow = null;
let tray = null;
const widgetWindows = new Map();

function createTaskbar() {
  const { width, height } = screen.getPrimaryDisplay().workAreaSize;
  const taskbarHeight = store.get('taskbarHeight') || 48;
  const position = store.get('taskbarPosition') || 'bottom';

  const pos = getTaskbarBounds(position, taskbarHeight, width, height);

  taskbarWindow = new BrowserWindow({
    x: pos.x,
    y: pos.y,
    width: pos.w,
    height: pos.h,
    frame: false,
    transparent: true,
    resizable: false,
    skipTaskbar: true,
    alwaysOnTop: true,
    hasShadow: false,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false
    }
  });

  taskbarWindow.loadFile(path.join(__dirname, 'src', 'taskbar', 'index.html'));
  taskbarWindow.setAlwaysOnTop(true, 'dock');
}

function getTaskbarBounds(position, barHeight, screenW, screenH) {
  const barWidth = Math.min(700, screenW - 40);
  const offsetX = Math.floor((screenW - barWidth) / 2);
  const barH = barHeight || 48;
  switch (position) {
    case 'top':    return { x: offsetX, y: 10, w: barWidth, h: barH };
    case 'left':   return { x: 10, y: Math.floor((screenH - barWidth) / 2), w: barH, h: barWidth };
    case 'right':  return { x: screenW - barH - 10, y: Math.floor((screenH - barWidth) / 2), w: barH, h: barWidth };
    default:       return { x: offsetX, y: screenH - barH - 10, w: barWidth, h: barH };
  }
}

function repositionTaskbar(position, barHeight) {
  if (!taskbarWindow || taskbarWindow.isDestroyed()) return;
  const { width: screenW, height: screenH } = screen.getPrimaryDisplay().workAreaSize;
  const pos = getTaskbarBounds(position || 'bottom', barHeight || 48, screenW, screenH);
  taskbarWindow.setBounds({ x: pos.x, y: pos.y, width: pos.w, height: pos.h }, true);
}

function createWallpaper() {
  if (wallpaperWindow && !wallpaperWindow.isDestroyed()) return;

  const { width, height } = screen.getPrimaryDisplay().bounds;

  wallpaperWindow = new BrowserWindow({
    x: 0, y: 0, width: width, height: height,
    frame: false, transparent: false, resizable: false,
    skipTaskbar: true, hasShadow: false,
    focusable: false,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false
    }
  });

  wallpaperWindow.loadFile(path.join(__dirname, 'src', 'wallpaper', 'index.html'));
  wallpaperWindow.setIgnoreMouseEvents(true, { forward: true });

  const wpPath = store.get('wallpaperPath');
  const wpVol = store.get('wallpaperVolume') || 0;

  wallpaperWindow.webContents.on('did-finish-load', () => {
    if (wpPath) {
      wallpaperWindow.webContents.send('load-wallpaper', { path: wpPath, volume: wpVol });
    }
    try {
      const hwnd = wallpaperWindow.getNativeWindowHandle();
      const { embedBehindDesktop } = require('./src/wallpaper/windows-desktop');
      const ok = embedBehindDesktop(hwnd);
      if (!ok) {
        wallpaperWindow.setAlwaysOnTop(true, 'desktop');
      }
    } catch (e) {
      console.error('Wallpaper embed failed, using fallback:', e.message);
      wallpaperWindow.setAlwaysOnTop(true, 'desktop');
    }
  });
}

function sendToWallpaper(channel, ...args) {
  if (wallpaperWindow && !wallpaperWindow.isDestroyed()) {
    wallpaperWindow.webContents.send(channel, ...args);
  }
}

function broadcastWallpaperToAllWindows() {
  const wpPath = store.get('wallpaperPath');
  const wpVol = store.get('wallpaperVolume') || 0;
  if (wpPath) {
    sendToWallpaper('load-wallpaper', { path: wpPath, volume: wpVol });
  }
}

function createLauncher() {
  if (launcherWindow) {
    launcherWindow.focus();
    return;
  }

  const { width, height } = screen.getPrimaryDisplay().workAreaSize;

  launcherWindow = new BrowserWindow({
    x: 0,
    y: Math.floor(height * 0.1),
    width: Math.floor(width * 0.45),
    height: Math.floor(height * 0.75),
    frame: false,
    backgroundColor: '#18181c',
    resizable: false,
    skipTaskbar: true,
    alwaysOnTop: true,
    show: false,
    hasShadow: false,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false
    }
  });

  launcherWindow.loadFile(path.join(__dirname, 'src', 'launcher', 'index.html'));
  launcherWindow.on('blur', () => {
    launcherWindow.hide();
  });
}

function createSettings() {
  if (settingsWindow) {
    settingsWindow.focus();
    return;
  }

  settingsWindow = new BrowserWindow({
    width: 700,
    height: 500,
    frame: false,
    backgroundColor: '#1e1e23',
    resizable: false,
    skipTaskbar: true,
    alwaysOnTop: true,
    show: false,
    hasShadow: false,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false
    }
  });

  settingsWindow.loadFile(path.join(__dirname, 'src', 'settings', 'index.html'));
  settingsWindow.on('blur', () => {
    settingsWindow.hide();
  });
}

function createWidget(widgetId) {
  const widgetConfigs = {
    'clock': { width: 220, height: 200, title: 'Saat & Takvim' },
    'system-monitor': { width: 240, height: 280, title: 'Sistem Monitörü' },
    'weather': { width: 260, height: 220, title: 'Hava Durumu' },
    'notes': { width: 280, height: 320, title: 'Notlar' },
    'music-player': { width: 300, height: 120, title: 'Müzik Player' }
  };

  const config = widgetConfigs[widgetId];
  if (!config) return;

  if (widgetWindows.has(widgetId)) {
    const existing = widgetWindows.get(widgetId);
    if (!existing.isDestroyed()) {
      existing.show();
      existing.focus();
    }
    return;
  }

  const savedPos = store.get(`widgetPositions.${widgetId}`);
  const { width: screenW, height: screenH } = screen.getPrimaryDisplay().workAreaSize;

  const win = new BrowserWindow({
    x: savedPos?.x || screenW - config.width - 20,
    y: savedPos?.y || 20 + (widgetWindows.size * 30),
    width: config.width,
    height: config.height,
    frame: false,
    backgroundColor: '#1e1e23',
    alwaysOnTop: true,
    skipTaskbar: true,
    resizable: false,
    show: false,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false
    }
  });

  win.loadFile(path.join(__dirname, 'src', 'widgets', widgetId, 'index.html'));

  win.webContents.on('did-finish-load', () => {
    win.webContents.setZoomFactor(1);
    win.webContents.setVisualZoomLevelLimits(1, 1);
  });

  win.once('ready-to-show', () => {
    win.show();
  });

  win.on('closed', () => {
    widgetWindows.delete(widgetId);
  });

  win.on('moved', () => {
    const [x, y] = win.getPosition();
    store.set(`widgetPositions.${widgetId}`, { x, y });
  });

  widgetWindows.set(widgetId, win);
}

function createTray() {
  const iconSize = 16;
  const icon = nativeImage.createEmpty();

  tray = new Tray(icon);

  const contextMenu = Menu.buildFromTemplate([
    { label: 'Widget\'ları Göster', click: () => showAllWidgets() },
    { label: 'Widget\'ları Gizle', click: () => hideAllWidgets() },
    { type: 'separator' },
    { label: 'Ayarlar', click: () => { createSettings(); settingsWindow.show(); } },
    { type: 'separator' },
    { label: 'Çıkış', click: () => app.quit() }
  ]);

  tray.setToolTip('detbar');
  tray.setContextMenu(contextMenu);

  tray.on('click', () => {
    if (launcherWindow && launcherWindow.isVisible()) {
      launcherWindow.hide();
    } else {
      createLauncher();
      launcherWindow.show();
    }
  });
}

function showAllWidgets() {
  const enabledWidgets = store.get('enabledWidgets');
  enabledWidgets.forEach(widgetId => createWidget(widgetId));
}

function hideAllWidgets() {
  widgetWindows.forEach((win) => {
    win.hide();
  });
}

function registerShortcuts() {
  globalShortcut.register('CommandOrControl+Alt+Space', () => {
    if (launcherWindow && launcherWindow.isVisible()) {
      launcherWindow.hide();
    } else {
      createLauncher();
      launcherWindow.show();
      launcherWindow.focus();
    }
  });

  globalShortcut.register('CommandOrControl+Alt+T', () => {
    if (settingsWindow && settingsWindow.isVisible()) {
      settingsWindow.hide();
    } else {
      createSettings();
      settingsWindow.show();
    }
  });

  globalShortcut.register('CommandOrControl+Alt+W', () => {
    const enabledWidgets = store.get('enabledWidgets');
    if (widgetWindows.size > 0) {
      hideAllWidgets();
    } else {
      showAllWidgets();
    }
  });
}

setupIpcHandlers(ipcMain, store, {
  createWidget,
  hideAllWidgets,
  showAllWidgets,
  createLauncher,
  createSettings,
  createWallpaper,
  widgetWindows,
  launcherWindow: () => launcherWindow,
  settingsWindow: () => settingsWindow,
  taskbarWindow: () => taskbarWindow,
  wallpaperWindow: () => wallpaperWindow,
  repositionTaskbar,
  setOpacity: (value) => {
    const opacity = value / 100;
    if (taskbarWindow && !taskbarWindow.isDestroyed()) taskbarWindow.setOpacity(opacity);
    if (launcherWindow && !launcherWindow.isDestroyed()) launcherWindow.setOpacity(opacity);
    if (settingsWindow && !settingsWindow.isDestroyed()) settingsWindow.setOpacity(opacity);
    widgetWindows.forEach((win) => {
      if (!win.isDestroyed()) win.setOpacity(opacity);
    });
  }
});

app.whenReady().then(() => {
  registerShortcuts();
  createTray();
  createTaskbar();
  createWallpaper();

  const enabledWidgets = store.get('enabledWidgets');
  enabledWidgets.forEach(widgetId => createWidget(widgetId));
});

app.on('window-all-closed', (e) => {
  e.preventDefault();
});

app.on('will-quit', () => {
  globalShortcut.unregisterAll();
});
