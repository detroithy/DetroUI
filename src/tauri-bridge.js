// Tauri v2 IPC bridge - simple version
(function() {
  var pending = [];
  var ready = false;
  var eventCbs = {};

  function tryInvoke() {
    try {
      var ii = window.__TAURI_INTERNALS__;
      if (ii && typeof ii.invoke === 'function') {
        window.__INVOKE__ = function(cmd, args) { return ii.invoke(cmd, args); };
        ready = true;
        for (var i = 0; i < pending.length; i++) {
          try { window.__INVOKE__(pending[i][0], pending[i][1]).then(pending[i][2]).catch(pending[i][3]); } catch(e) { pending[i][3](e); }
        }
        pending = [];
        return true;
      }
    } catch(e) {}
    return false;
  }

  if (!tryInvoke()) {
    var iv = setInterval(function() { if (tryInvoke()) clearInterval(iv); }, 100);
    setTimeout(function() { if (!ready) console.warn('Tauri IPC not available'); }, 5000);
  }

  function call(cmd, args) {
    var cname = cmd.replace(/([a-z])([A-Z])/g, '$1_$2').toLowerCase();
    if (ready) return window.__INVOKE__(cname, args || {});
    return new Promise(function(res, rej) { pending.push([cname, args || {}, res, rej]); });
  }

  function label() {
    var p = window.location.pathname;
    if (p.indexOf('/widgets/') >= 0) { var m = p.match(/\/widgets\/([^\/]+)/); if (m) return 'widget-' + m[1]; }
    if (p.indexOf('/settings/') >= 0) return 'settings';
    if (p.indexOf('/launcher/') >= 0) return 'launcher';
    if (p.indexOf('/taskbar/') >= 0) return 'taskbar';
    return null;
  }

  window.plasmaAPI = {
    getCpuLoad: function() { return call('getCpuLoad'); },
    getMemory: function() { return call('getMemory'); },
    getDisk: function() { return call('getDisk'); },
    getBattery: function() { return call('getBattery'); },
    getCpuTemp: function() { return call('getCpuTemp'); },
    getNetwork: function() { return call('getNetwork'); },
    getWeather: function(city) { return call('getWeather', { city: city }); },
    getInstalledApps: function() { return call('getInstalledApps'); },
    launchApp: function(path) { return call('launchApp', { path: path }); },
    getMediaSessions: function() { return call('getMediaSessions'); },
    mediaPlay: function(app) { return call('mediaPlay', { app: app }); },
    mediaPause: function(app) { return call('mediaPause', { app: app }); },
    mediaNext: function(app) { return call('mediaNext', { app: app }); },
    mediaPrev: function(app) { return call('mediaPrev', { app: app }); },
    mediaToggle: function() { return call('mediaToggle'); },
    mediaSeek: function(percent) { return call('mediaSeek', { percent: percent }); },
    mediaVolumeUp: function() { return call('mediaVolumeUp'); },
    mediaVolumeDown: function() { return call('mediaVolumeDown'); },
    mediaMute: function() { return call('mediaMute'); },
    createWidget: function(id) { return call('createWidget', { id: id }); },
    closeWidget: function(id) { return call('closeWidget', { id: id }); },
    getWidgetWindows: function() { return call('getWidgetWindows'); },
    openLauncher: function() { return call('openLauncherWindow'); },
    closeLauncher: function() { return call('closeLauncherWindow'); },
    toggleLauncher: function() { return call('toggleLauncherWindow'); },
    openSettings: function() { return call('openSettingsWindow'); },
    closeSettings: function() { return call('closeSettingsWindow'); },
    getStoreValue: function(key) { return call('getStoreValue', { key: key }); },
    setStoreValue: function(key, value) { return call('setStoreValue', { key: key, value: value }); },
    getTheme: function() { return call('getTheme'); },
    setTheme: function(theme) { return call('setTheme', { theme: theme }); },
    getAccentColor: function() { return call('getAccentColor'); },
    setAccentColor: function(color) { return call('setAccentColor', { color: color }); },
    setOpacity: function(value) { return call('setStoreValue', { key: 'opacity', value: value }); },
    getNotes: function() { return call('getNotes'); },
    saveNotes: function(notes) { return call('saveNotes', { notes: notes }); },
    setWallpaper: function(path) { return call('setStoreValue', { key: 'wallpaperPath', value: path }); },
    stopWallpaper: function() { return call('setStoreValue', { key: 'wallpaperPath', value: '' }); },
    setWallpaperVolume: function(vol) { return call('setStoreValue', { key: 'wallpaperVolume', value: vol }); },
    setWallpaperPlayPause: function(play) { return call('setStoreValue', { key: 'wallpaperPlayPause', value: play }); },
    pickWallpaperFile: function() { return call('pickFile', { title: 'Video Seç', filter: 'Video|*.mp4;*.webm;*.avi;*.mkv;*.mov|All|*.*' }); },
    pickFolder: function(title) { return call('pickFolder', { title: title || 'Klasör Seç' }); },
    createWallpaperWindow: function() { return call('createWallpaperWindow'); },
    startVideoServer: function(path) { return call('startVideoServer', { path: path }); },
    stopVideoServer: function() { return call('stopVideoServer'); },
    setWindowSize: function(label, width, height) { return call('setWindowSize', { label: label, width: width, height: height }); },
    getDesktops: function() { return call('getDesktops'); },
    switchDesktop: function(id) { return call('switchDesktop', { id: id }); },
    createDesktop: function(name) { return call('createDesktop', { name: name }); },
    deleteDesktop: function(id) { return call('deleteDesktop', { id: id }); },
    renameDesktop: function(id, name) { return call('renameDesktop', { id: id, name: name }); },
    enumOpenWindows: function() { return call('enumOpenWindows'); },
    moveAppWindow: function(hwnd, x, y, w, h) { return call('moveAppWindow', { hwnd: hwnd, x: x, y: y, w: w, h: h }); },
    showAppWindow: function(hwnd) { return call('showAppWindow', { hwnd: hwnd }); },
    hideAppWindow: function(hwnd) { return call('hideAppWindow', { hwnd: hwnd }); },
    addDesktopShortcut: function(desktopId, name, path, icon) { return call('addDesktopShortcut', { desktopId: desktopId, name: name, path: path, icon: icon }); },
    updateShortcutPosition: function(desktopId, index, x, y) { return call('updateShortcutPosition', { desktopId: desktopId, index: index, x: x, y: y }); },
    deleteDesktopShortcut: function(desktopId, index) { return call('deleteDesktopShortcut', { desktopId: desktopId, index: index }); },
    tileCurrentDesktop: function() { return call('tileCurrentDesktop'); },
    promoteMaster: function(hwnd) { return call('promoteMaster', { hwnd: hwnd }); },
    moveWindowToDesktop: function(hwnd, toId, remember) { return call('moveWindowToDesktop', { hwnd: hwnd, toId: toId, remember: remember || false }); },
    getAutoAssignWindows: function() { return call('getAutoAssignWindows'); },
    setAutoAssignWindows: function(value) { return call('setAutoAssignWindows', { value: value }); },
    getAppDesktopMap: function() { return call('getAppDesktopMap'); },
    setAppDesktopMap: function(map) { return call('setAppDesktopMap', { map: map }); },
    getWindowUnderCursor: function() { return call('getWindowUnderCursor'); },
    getCurrentMousePos: function() { return call('getCurrentMousePos'); },
    removeFromAllDesktops: function(hwnd) { return call('removeWindowFromAllDesktops', { hwnd: hwnd }); },
    screenInformation: function() { return call('screenInformation'); },
    moveWindow: function(x, y) { var l = label(); if (l) return call('moveWindow', { label: l, x: x, y: y }); },
    getWindowPosition: function() { var l = label(); if (l) return call('getWindowPosition', { label: l }); return Promise.resolve({ x: 0, y: 0 }); },
    setWindowPosition: function(x, y) { return window.plasmaAPI.moveWindow(x, y); },
    onThemeChanged: function(cb) {},
    onWidgetConfigChanged: function(cb) {},
    onTaskbarSettingsChanged: function(cb) {},
    onSystemUpdate: function(cb) {},
    onLoadWallpaper: function(cb) {},
    onWallpaperVolume: function(cb) {},
    onWallpaperPlayPause: function(cb) {},
    onWallpaperStop: function(cb) {},
  };
})();
