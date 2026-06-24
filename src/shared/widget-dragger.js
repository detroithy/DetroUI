// Ortak Sürükleme Utility - Tüm widget'lar tarafından kullanılır
// Bu dosya her widget'ın renderer.js'inde require edilmeli

class WidgetDragger {
  constructor(options = {}) {
    this.isDragging = false;
    this.startX = 0;
    this.startY = 0;
    this.windowX = 0;
    this.windowY = 0;
    this.dragHandle = null;
    this.widgetEl = null;
    this.enabled = true;
    this.onDragStart = null;
    this.onDragEnd = null;

    // Seçenekler
    this.gridSnap = options.gridSnap || false;       // Grid'e hizalama
    this.gridSize = options.gridSize || 20;          // Grid boyutu (px)
    this.screenEdges = options.screenEdges || true;  // Ekran kenarlarında dur
    this.edgeMargin = options.edgeMargin || 10;      // Kenar boşluğu (px)
    this.showTooltip = options.showTooltip || true;  // Sürükleme ipucu göster
  }

  init(widgetSelector = '.widget', handleSelector = '.drag-handle, .widget-header') {
    this.widgetEl = document.querySelector(widgetSelector);
    this.dragHandle = document.querySelector(handleSelector);

    if (!this.dragHandle) {
      this.dragHandle = this.widgetEl;
    }

    if (!this.dragHandle) return;

    this.setupEventListeners();
    this.createTooltip();
  }

  createTooltip() {
    if (!this.showTooltip) return;

    this.tooltip = document.createElement('div');
    this.tooltip.className = 'drag-tooltip';
    this.tooltip.innerHTML = 'Widget\'ı sürükleyerek konumlandırın';
    document.body.appendChild(this.tooltip);
  }

  setupEventListeners() {
    // Mouse olayları
    this.dragHandle.addEventListener('mousedown', (e) => this.startDrag(e));
    document.addEventListener('mousemove', (e) => this.onDrag(e));
    document.addEventListener('mouseup', () => this.endDrag());

    // Touch olayları (mobil destek)
    this.dragHandle.addEventListener('touchstart', (e) => {
      e.preventDefault();
      this.startDrag(e.touches[0]);
    }, { passive: false });
    document.addEventListener('touchmove', (e) => {
      if (this.isDragging) {
        e.preventDefault();
        this.onDrag(e.touches[0]);
      }
    }, { passive: false });
    document.addEventListener('touchend', () => this.endDrag());

    // Sürükleme sırasında cursor değiştir
    this.dragHandle.style.cursor = 'grab';
    this.dragHandle.addEventListener('mouseenter', () => {
      if (!this.isDragging) this.dragHandle.style.cursor = 'grab';
    });
    this.dragHandle.addEventListener('mouseleave', () => {
      if (!this.isDragging) this.dragHandle.style.cursor = 'default';
    });
  }

  async startDrag(e) {
    if (!this.enabled) return;

    this.isDragging = true;
    this.startX = e.screenX;
    this.startY = e.screenY;

    // Mevcut pencere pozisyonunu al
    const pos = await window.plasmaAPI.getWindowPosition();
    this.windowX = pos.x;
    this.windowY = pos.y;

    // Stil güncelle
    this.dragHandle.style.cursor = 'grabbing';
    if (this.widgetEl) {
      this.widgetEl.classList.add('dragging');
    }

    // Sürükleme ipucunu göster
    if (this.tooltip && this.showTooltip) {
      this.tooltip.classList.add('visible');
    }

    // Seçimi devre dışı bırak
    document.body.style.userSelect = 'none';
    document.body.style.cursor = 'grabbing';

    if (this.onDragStart) this.onDragStart();
  }

  onDrag(e) {
    if (!this.isDragging) return;

    const deltaX = e.screenX - this.startX;
    const deltaY = e.screenY - this.startY;

    let newX = this.windowX + deltaX;
    let newY = this.windowY + deltaY;

    // Grid snap
    if (this.gridSnap) {
      newX = Math.round(newX / this.gridSize) * this.gridSize;
      newY = Math.round(newY / this.gridSize) * this.gridSize;
    }

    // Ekran kenarı kontrolü
    if (this.screenEdges) {
      const screenInfo = this.getScreenBounds();
      const widgetWidth = this.widgetEl ? this.widgetEl.offsetWidth : 200;
      const widgetHeight = this.widgetEl ? this.widgetEl.offsetHeight : 200;

      // Sol kenar
      if (newX < screenInfo.x + this.edgeMargin) {
        newX = screenInfo.x + this.edgeMargin;
      }
      // Sağ kenar
      if (newX + widgetWidth > screenInfo.x + screenInfo.width - this.edgeMargin) {
        newX = screenInfo.x + screenInfo.width - widgetWidth - this.edgeMargin;
      }
      // Üst kenar
      if (newY < screenInfo.y + this.edgeMargin) {
        newY = screenInfo.y + this.edgeMargin;
      }
      // Alt kenar
      if (newY + widgetHeight > screenInfo.y + screenInfo.height - this.edgeMargin) {
        newY = screenInfo.y + screenInfo.height - widgetHeight - this.edgeMargin;
      }
    }

    // Pencereyi taşı
    window.plasmaAPI.moveWindow(newX, newY);

    // Tooltip pozisyonunu güncelle
    if (this.tooltip && this.showTooltip) {
      this.tooltip.style.left = `${e.clientX}px`;
      this.tooltip.style.top = `${e.clientY + 20}px`;
    }
  }

  endDrag() {
    if (!this.isDragging) return;

    this.isDragging = false;

    // Stil güncelle
    this.dragHandle.style.cursor = 'grab';
    if (this.widgetEl) {
      this.widgetEl.classList.remove('dragging');
    }

    // Tooltip'i gizle
    if (this.tooltip && this.showTooltip) {
      this.tooltip.classList.remove('visible');
    }

    // Seçimi geri aç
    document.body.style.userSelect = '';
    document.body.style.cursor = '';

    if (this.onDragEnd) this.onDragEnd();
  }

  getScreenBounds() {
    // Varsayılan ekran boyutları (gerçek API'den alınabilir)
    return {
      x: 0,
      y: 0,
      width: window.screen.width || 1920,
      height: window.screen.height || 1080
    };
  }

  enable() {
    this.enabled = true;
  }

  disable() {
    this.enabled = false;
  }

  setGridSnap(enabled, size = 20) {
    this.gridSnap = enabled;
    this.gridSize = size;
  }

  destroy() {
    if (this.tooltip && this.tooltip.parentNode) {
      this.tooltip.parentNode.removeChild(this.tooltip);
    }
  }
}

// Global olarak erişilebilir yap
if (typeof window !== 'undefined') {
  window.WidgetDragger = WidgetDragger;
}
