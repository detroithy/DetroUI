let settings = { theme: 'dark', accentColor: '#00d4ff', taskbarPosition: 'bottom', taskbarHeight: 48, opacity: 88 };
let lastSettings = {};

document.addEventListener('DOMContentLoaded', async function() {
  updateClock();
  setInterval(updateClock, 1000);
  setupEventListeners();
  pollSettings();
});

function pollSettings() {
  setInterval(async function() {
    try {
      var theme = await window.plasmaAPI.getTheme();
      var accent = await window.plasmaAPI.getAccentColor();
      var pos = await window.plasmaAPI.getStoreValue('taskbarPosition') || 'bottom';
      var h = parseInt(await window.plasmaAPI.getStoreValue('taskbarHeight')) || 48;
      var op = parseFloat(await window.plasmaAPI.getStoreValue('opacity')) || 88;
      var changed = lastSettings.theme !== theme || lastSettings.accent !== accent ||
                    lastSettings.pos !== pos || lastSettings.height !== h || lastSettings.opacity !== op;
      if (changed) {
        lastSettings = { theme, accent, pos, height: h, opacity: op };
        settings = { theme, accentColor: accent, taskbarPosition: pos, taskbarHeight: h, opacity: op };
        applyTheme();
        applyTaskbarPosition();
        if (lastSettings.pos !== pos || lastSettings.height !== h) {
          try { await window.plasmaAPI.setStoreValue('taskbarPosPending', pos); } catch(e) {}
        }
        document.body.style.opacity = (op / 100);
        // Taskbar config'ini Rust'a bildir
        try { await window.__TAURI_INTERNALS__.invoke('set_taskbar_config', { position: pos, height: h }); } catch(e) {}
      }
      // Widget states
      try {
        var wins = await window.plasmaAPI.getWidgetWindows();
        document.querySelectorAll('.widget-toggle').forEach(function(btn) {
          var w = wins.find(function(x) { return x.id === btn.dataset.widget && x.visible; });
          btn.classList.toggle('active', !!w);
        });
      } catch(e) {}
    } catch(e) {}
  }, 1500);
}

function applyTheme() {
  var root = document.documentElement;
  var a = settings.accentColor || '#00d4ff';
  root.style.setProperty('--accent', a);
  var themes = {
    dark:     { bg: '#18181c', card: '#1e1e23', text: '#e0e0e0', textDim: '#b0b0b0' },
    light:    { bg: '#e8e8e8', card: '#f5f5f5', text: '#1a1a1a', textDim: '#555' },
    catppuccin: { bg: '#181824', card: '#1e1e2e', text: '#cdd6f4', textDim: '#a6adc8' },
    nord:     { bg: '#2e3440', card: '#3b4252', text: '#eceff4', textDim: '#d8dee9' },
    dracula:  { bg: '#21222c', card: '#282a36', text: '#f8f8f2', textDim: '#6272a4' }
  };
  var t = themes[settings.theme] || themes.dark;
  root.style.setProperty('--bg', t.bg);
  root.style.setProperty('--bg-card', t.card);
  root.style.setProperty('--text', t.text);
  root.style.setProperty('--text-dim', t.textDim);
}

function applyTaskbarPosition() {
  var taskbar = document.querySelector('.taskbar');
  if (!taskbar) return;
  var pos = settings.taskbarPosition || 'bottom';
  taskbar.className = 'taskbar taskbar-' + pos;
}

function updateClock() {
  var now = new Date();
  var el = document.getElementById('clock');
  if (el) el.textContent = now.toLocaleDateString('tr-TR', { day: 'numeric', month: 'short', weekday: 'short' }) + '  ' + now.toLocaleTimeString('tr-TR', { hour: '2-digit', minute: '2-digit' });
}

function setupEventListeners() {
  var si = document.getElementById('searchInput');
  var lastOpen = 0;
  if (si) si.addEventListener('click', function() {
    var now = Date.now();
    if (now - lastOpen < 800) return;
    lastOpen = now;
    window.plasmaAPI.openLauncher();
    si.blur();
  });

  document.querySelectorAll('.app-button').forEach(function(btn) {
    btn.addEventListener('click', function() {
      var paths = { explorer: 'explorer.exe', browser: 'start msedge:', terminal: 'wt.exe' };
      if (paths[btn.dataset.app]) window.plasmaAPI.launchApp(paths[btn.dataset.app]);
    });
  });

  document.querySelectorAll('.widget-toggle').forEach(function(btn) {
    btn.addEventListener('click', async function() {
      var id = btn.dataset.widget;
      if (btn.classList.contains('active')) { window.plasmaAPI.closeWidget(id); btn.classList.remove('active'); }
      else { window.plasmaAPI.createWidget(id); btn.classList.add('active'); }
    });
  });

  var sb = document.getElementById('settingsBtn');
  if (sb) sb.addEventListener('click', function() { window.plasmaAPI.openSettings(); });

  var ni = document.getElementById('networkIcon');
  if (ni) ni.addEventListener('click', function() { window.plasmaAPI.launchApp('start ms-settings:network-wifi'); });

  var vi = document.getElementById('volumeIcon');
  if (vi) vi.addEventListener('click', function() { window.plasmaAPI.launchApp('start ms-settings:sound'); });
}
