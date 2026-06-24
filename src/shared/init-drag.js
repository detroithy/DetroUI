function initDrag(widgetSelector) {
  const widget = document.querySelector(widgetSelector);
  const dragBtn = document.getElementById('dragBtn');
  const closeBtn = document.getElementById('closeBtn');

  if (!widget) return;

  // SURUKLEME - saf JS ile, -webkit-app-region KULLANMA
  if (dragBtn) {
    let isDragging = false;
    let startX = 0, startY = 0, winX = 0, winY = 0;

    dragBtn.addEventListener('mousedown', async (e) => {
      isDragging = true;
      startX = e.screenX;
      startY = e.screenY;
      try {
        const pos = await window.plasmaAPI.getWindowPosition();
        winX = pos.x;
        winY = pos.y;
      } catch (err) {}
      dragBtn.classList.add('active');
      e.preventDefault();
    });

    document.addEventListener('mousemove', (e) => {
      if (!isDragging) return;
      window.plasmaAPI.moveWindow(winX + (e.screenX - startX), winY + (e.screenY - startY));
    });

    document.addEventListener('mouseup', () => {
      if (!isDragging) return;
      isDragging = false;
      dragBtn.classList.remove('active');
    });
  }

  // KAPAT butonu
  if (closeBtn) {
    closeBtn.addEventListener('click', () => {
      if (window.plasmaAPI && window.plasmaAPI.closeWidget) {
        window.plasmaAPI.closeWidget(widget.dataset.widgetId);
      }
    });
  }

  // TEMA
  if (window.plasmaAPI && window.plasmaAPI.onThemeChanged) {
    window.plasmaAPI.onThemeChanged((data) => {
      applyWidgetTheme(data.theme, data.accentColor);
    });
  }
  loadAndApplyTheme();
}

async function loadAndApplyTheme() {
  try {
    const theme = await window.plasmaAPI.getTheme();
    const accent = await window.plasmaAPI.getAccentColor();
    applyWidgetTheme(theme, accent);
  } catch (e) {}
}

function applyWidgetTheme(theme, accentColor) {
  if (accentColor) document.documentElement.style.setProperty('--accent', accentColor);
  const themes = {
    dark: { bg: '#1e1e23', card: '#252529' },
    light: { bg: '#e8e8e8', card: '#f5f5f5' },
    catppuccin: { bg: '#1e1e2e', card: '#252536' },
    nord: { bg: '#2e3440', card: '#3b4252' },
    dracula: { bg: '#282a36', card: '#343746' }
  };
  const t = themes[theme] || themes.dark;
  document.body.style.background = t.bg;
  const card = document.querySelector('.widget');
  if (card) card.style.background = t.card;
}
