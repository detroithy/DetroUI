# DetroUI

Windows 10/11 için geliştirilmiş, yüksek performanslı masaüstü ortamı. Tauri v2 + WebView2 tabanlı, minimal ve özelleştirilebilir.

## Özellikler

### Görev Çubuğu (detbar)
- **Pill-shaped tasarım** - Köşeli, kompakt, merkezi konumlandırma
- **Saat** - Gerçek zamanlı
- **Widget Toggle** - Widget'ları aç/kapat
- **Sistem Tepsisi** - Arka plan uygulamaları
- **Özelleştirme** - Renk, pozisyon, yükseklik, opaklık

### Sanal Masaüstleri
- **3 Varsayılan Masaüstü** - Ana, İş, Eğlence
- **Ctrl+Fare Tekerleği** - Masaüstleri arası geçiş
- **Masaüstü Atama** - Pencereleri sağ tıkla ile masaüstüne ata
- **Pencere Seç (Tıkla)** - Crosshair modu ile tıklayarak seç
- **Sürükle-Bırak Transfer** - Ekran kenarına sürükle → otomatik taşı
- **Snap Devre Dışı** - Kenar transferi için Windows Snap kapatıldı

### Tiling Pencere Yöneticisi
- **Master-Stack** - Sol %60 (master) + Sağ %40 (stack)
- **Otomatik Tiling** - Masaüstüne atanan pencereleri otomatik diz
- **Kısayollar** - Win+Shift+ok ile tiling kontrolü (yakında)

### Widget'lar
- **Saat & Takvim** - Analog/dijital görünüm
- **Sistem Monitörü** - CPU, RAM, disk, ağ kullanımı
- **Hava Durumu** - Gerçek zamanlı hava durumu ve tahmin
- **Notlar** - Hızlı not alma ve düzenleme
- **Müzik Player** - Medya kontrolü
- **Sürükleme** - Widget'ları fare ile taşı, boyutlandır
- **Tema Uyumluluğu** - Tüm tema/renk/opacity değişiklikleri anında uygulanır

### Animasyonlu Duvar Kağıdı
- **Video Duvar Kağıdı** - HTTP sunucusu üzerinden video dosyaları
- **WorkerW Embed** - Masaüstü simgelerinin arkasında çalışır
- **Kontroller** - Oynat/duraklat, ses, ileri/geri
- **Otomatik Algılama** - Mevcut sunucu varsa yeniden başlatmaz
- **Debug Overlay** - Yeşil metin ile durum göstergesi

### Uygulama Başlatıcı
- **Arama** - Sistem komutları (cmd, powershell, notepad, chrome vb.)
- **Güç Menüsü** - Kapat, yeniden başlat, uyku
- **Kapat Düğmesi** - ✕ ile kapat
- **Escape** - Tuşu ile kapat

### Ayarlar Paneli
- **5 Tema** - Dark, Light, Catppuccin, Nord, Dracula
- **Vurgu Renkleri** - 6 farklı renk seçeneği
- **Opaklık** - Global pencere opaklık ayarı
- **Görev Çubuğu** - Pozisyon, yükseklik, renk
- **Duvar Kağıdı** - Video dosyası seçimi, ses kontrolü

## Kurulum

```bash
# Gereksinimler
# - Node.js 18+
# - Rust (rustup)
# - Windows SDK 10.0.22621.0
# - Visual Studio Build Tools

# Bağımlılıkları yükle
npm install

# Geliştirme modunda çalıştır
cargo build --manifest-path src-tauri/Cargo.toml
cargo run --manifest-path src-tauri/Cargo.toml

# Production build
cargo build --release --manifest-path src-tauri/Cargo.toml
```

## Kısayollar

| Kısayol | İşlev |
|---------|-------|
| `Ctrl+Fare Tekerleği` | Masaüstleri arası geçiş |
| `Sağ Tık` | Pencere menüsü (masaüstü ata) |
| `Win+Space` | Uygulama başlatıcı |

## Teknolojiler

- **Tauri v2** - Native framework (Rust backend + WebView2 frontend)
- **Rust** - Sistem seviyesi işlemler, FFI, pencere yönetimi
- **WebView2** - Chromium tabanlı webview
- **HTML/CSS/JS** - Frontend arayüzü
- **Windows API** - WorkerW embed, pencere manipülasyonu, sanal masaüstü

## Yapılandırma

Ayarlar panelinden tema, vurgu rengi, widget ayarları ve daha fazlasını özelleştirebilirsiniz. Tüm ayarlar `%APPDATA%/detbar/config.json` dosyasına kaydedilir.

## Lisans

MIT
