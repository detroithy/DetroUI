let currentSettings = {};

document.addEventListener('DOMContentLoaded', async () => {
  try {
    await loadSettings();
  } catch(e) {
    console.warn('Settings load failed, using defaults:', e);
    currentSettings = {
      theme: 'dark',
      accentColor: '#00d4ff',
      weatherCity: 'istanbul',
      enabledWidgets: ['clock', 'system-monitor'],
      taskbarPosition: 'bottom',
      taskbarHeight: 48,
      opacity: 88
    };
  }
  setupEventListeners();

  document.getElementById('closeBtn').addEventListener('click', function() {
    window.plasmaAPI.closeSettings();
  });

  try {
    window.plasmaAPI.onThemeChanged(function(data) {
      currentSettings.theme = data.theme;
      currentSettings.accentColor = data.accentColor;
      applyCurrentSettings();
    });
  } catch(e) {}
});

async function loadSettings() {
  currentSettings = {
    theme: await window.plasmaAPI.getTheme() || 'dark',
    accentColor: await window.plasmaAPI.getAccentColor() || '#00d4ff',
    weatherCity: await window.plasmaAPI.getStoreValue('weatherCity') || 'istanbul',
    enabledWidgets: await window.plasmaAPI.getStoreValue('enabledWidgets') || ['clock', 'system-monitor'],
    taskbarPosition: await window.plasmaAPI.getStoreValue('taskbarPosition') || 'bottom',
    taskbarHeight: await window.plasmaAPI.getStoreValue('taskbarHeight') || 48,
    opacity: await window.plasmaAPI.getStoreValue('opacity') || 88
  };
  applyCurrentSettings();
}

function applyCurrentSettings() {
  var root = document.documentElement;
  var a = currentSettings.accentColor || '#00d4ff';
  root.style.setProperty('--accent', a);

  var themes = {
    dark:     { bg: '#1e1e23', card: '#252529', text: '#fff', textDim: 'rgba(255,255,255,0.7)' },
    light:    { bg: '#e8e8e8', card: '#f5f5f5', text: '#1a1a1a', textDim: '#555' },
    catppuccin: { bg: '#1e1e2e', card: '#252536', text: '#cdd6f4', textDim: '#a6adc8' },
    nord:     { bg: '#2e3440', card: '#3b4252', text: '#eceff4', textDim: '#d8dee9' },
    dracula:  { bg: '#282a36', card: '#343746', text: '#f8f8f2', textDim: '#6272a4' }
  };
  var t = themes[currentSettings.theme] || themes.dark;
  root.style.setProperty('--bg', t.bg);
  root.style.setProperty('--bg-card', t.card);
  root.style.setProperty('--text', t.text);
  root.style.setProperty('--text-dim', t.textDim);

  document.querySelectorAll('.theme-btn').forEach(function(btn) {
    btn.classList.toggle('active', btn.dataset.theme === currentSettings.theme);
  });

  document.querySelectorAll('.color-btn').forEach(function(btn) {
    btn.classList.toggle('active', btn.dataset.color === currentSettings.accentColor);
  });

  document.querySelectorAll('.toggle input[data-widget]').forEach(function(toggle) {
    toggle.checked = currentSettings.enabledWidgets.includes(toggle.dataset.widget);
  });

  document.querySelectorAll('.position-btn').forEach(function(btn) {
    btn.classList.toggle('active', btn.dataset.position === currentSettings.taskbarPosition);
  });

  var heightSlider = document.getElementById('taskbarHeight');
  if (heightSlider) {
    heightSlider.value = currentSettings.taskbarHeight;
    var hv = document.getElementById('taskbarHeightValue');
    if (hv) hv.textContent = currentSettings.taskbarHeight + 'px';
  }

  var opacitySlider = document.getElementById('opacitySlider');
  if (opacitySlider) {
    opacitySlider.value = currentSettings.opacity;
    var ov = document.getElementById('opacityValue');
    if (ov) ov.textContent = currentSettings.opacity + '%';
  }

  var citySelect = document.getElementById('weatherCity');
  if (citySelect) citySelect.value = currentSettings.weatherCity;
}

async function setupEventListeners() {
  document.querySelectorAll('.menu-item').forEach(function(item) {
    item.addEventListener('click', function() {
      document.querySelectorAll('.menu-item').forEach(function(i) { i.classList.remove('active'); });
      document.querySelectorAll('.panel-section').forEach(function(s) { s.classList.remove('active'); });
      item.classList.add('active');
      var section = document.getElementById(item.dataset.section);
      if (section) section.classList.add('active');
    });
  });

  document.querySelectorAll('.theme-btn').forEach(function(btn) {
    btn.addEventListener('click', async function() {
      document.querySelectorAll('.theme-btn').forEach(function(b) { b.classList.remove('active'); });
      btn.classList.add('active');
      currentSettings.theme = btn.dataset.theme;
      try { await window.plasmaAPI.setTheme(currentSettings.theme); } catch(e) { console.warn('setTheme failed', e); }
      applyCurrentSettings();
    });
  });

  document.querySelectorAll('.color-btn').forEach(function(btn) {
    btn.addEventListener('click', async function() {
      document.querySelectorAll('.color-btn').forEach(function(b) { b.classList.remove('active'); });
      btn.classList.add('active');
      currentSettings.accentColor = btn.dataset.color;
      try { await window.plasmaAPI.setAccentColor(currentSettings.accentColor); } catch(e) { console.warn('setAccentColor failed', e); }
      applyCurrentSettings();
    });
  });

  var opacitySlider = document.getElementById('opacitySlider');
  if (opacitySlider) {
    opacitySlider.addEventListener('input', function(e) {
      var val = parseInt(e.target.value);
      document.getElementById('opacityValue').textContent = val + '%';
      currentSettings.opacity = val;
      window.plasmaAPI.setOpacity(val);
      document.body.style.opacity = (val / 100);
    });
  }

  document.querySelectorAll('.toggle input[data-widget]').forEach(function(toggle) {
    toggle.addEventListener('change', async function() {
      var widgetId = toggle.dataset.widget;
      if (toggle.checked) {
        if (!currentSettings.enabledWidgets.includes(widgetId)) {
          currentSettings.enabledWidgets.push(widgetId);
        }
        window.plasmaAPI.createWidget(widgetId);
      } else {
        currentSettings.enabledWidgets = currentSettings.enabledWidgets.filter(function(w) { return w !== widgetId; });
        window.plasmaAPI.closeWidget(widgetId);
      }
      await window.plasmaAPI.setStoreValue('enabledWidgets', currentSettings.enabledWidgets);
    });
  });

  document.querySelectorAll('.position-btn').forEach(function(btn) {
    btn.addEventListener('click', async function() {
      document.querySelectorAll('.position-btn').forEach(function(b) { b.classList.remove('active'); });
      btn.classList.add('active');
      currentSettings.taskbarPosition = btn.dataset.position;
      await window.plasmaAPI.setStoreValue('taskbarPosition', currentSettings.taskbarPosition);
      try { await window.__TAURI_INTERNALS__.invoke('set_taskbar_config', { position: currentSettings.taskbarPosition, height: currentSettings.taskbarHeight }); } catch(e) {}
    });
  });

  var heightSlider = document.getElementById('taskbarHeight');
  if (heightSlider) {
    heightSlider.addEventListener('input', function(e) {
      document.getElementById('taskbarHeightValue').textContent = e.target.value + 'px';
    });
    heightSlider.addEventListener('change', async function(e) {
      currentSettings.taskbarHeight = parseInt(e.target.value);
      await window.plasmaAPI.setStoreValue('taskbarHeight', currentSettings.taskbarHeight);
      try { await window.__TAURI_INTERNALS__.invoke('set_taskbar_config', { position: currentSettings.taskbarPosition, height: currentSettings.taskbarHeight }); } catch(e) {}
    });
  }

  var citySelect = document.getElementById('weatherCity');
  if (citySelect) {
    citySelect.addEventListener('change', async function(e) {
      currentSettings.weatherCity = e.target.value;
      await window.plasmaAPI.setStoreValue('weatherCity', currentSettings.weatherCity);
    });
  }

  // Wallpaper
  var wpPath = await window.plasmaAPI.getStoreValue('wallpaperPath');
  var wpVol = await window.plasmaAPI.getStoreValue('wallpaperVolume');
  if (wpPath) {
    var pathEl = document.getElementById('wallpaperPath');
    if (pathEl) pathEl.textContent = wpPath;
  }
  var volSlider = document.getElementById('wallpaperVolume');
  if (volSlider) {
    volSlider.value = (wpVol || 0) * 100;
    var volVal = document.getElementById('wallpaperVolumeValue');
    if (volVal) volVal.textContent = Math.round((wpVol || 0) * 100) + '%';
  }

  document.getElementById('pickWallpaperBtn').addEventListener('click', async function() {
    console.log('[settings] Video Sec tiklandi');
    try {
      var filePath = await window.plasmaAPI.pickWallpaperFile();
      console.log('[settings] Secilen yol:', filePath);
      if (filePath) {
        console.log('[settings] Wallpaper penceresi olusturuluyor...');
        await window.plasmaAPI.createWallpaperWindow();
        console.log('[settings] Video sunucusu baslatiliyor...');
        var port = await window.plasmaAPI.startVideoServer(filePath);
        console.log('[settings] Video sunucusu port:', port);
        await window.plasmaAPI.setWallpaper(filePath);
        console.log('[settings] Yol kaydedildi:', filePath);
        var pathEl = document.getElementById('wallpaperPath');
        if (pathEl) pathEl.textContent = filePath;
      } else {
        console.log('[settings] Dosya secilmedi (null)');
      }
    } catch(e) {
      console.error('[settings] Wallpaper hatasi:', e);
    }
  });

  document.getElementById('stopWallpaperBtn').addEventListener('click', async function() {
    await window.plasmaAPI.stopVideoServer();
    window.plasmaAPI.stopWallpaper();
    var pathEl = document.getElementById('wallpaperPath');
    if (pathEl) pathEl.textContent = '';
  });

  var wpVolSlider = document.getElementById('wallpaperVolume');
  if (wpVolSlider) {
    wpVolSlider.addEventListener('input', function(e) {
      var val = parseInt(e.target.value);
      document.getElementById('wallpaperVolumeValue').textContent = val + '%';
      window.plasmaAPI.setWallpaperVolume(val / 100);
    });
  }

  document.getElementById('wallpaperPlayBtn').addEventListener('click', function() {
    window.plasmaAPI.setWallpaperPlayPause(true);
  });

  document.getElementById('wallpaperPauseBtn').addEventListener('click', function() {
    window.plasmaAPI.setWallpaperPlayPause(false);
  });

  document.addEventListener('keydown', function(e) {
    if (e.key === 'Escape') window.plasmaAPI.closeSettings();
  });

  // Opacity polling
  setInterval(async () => {
    try {
      var op = await window.plasmaAPI.getStoreValue('opacity');
      if (op) document.body.style.opacity = (op / 100);
    } catch(e) {}
  }, 2000);
}
