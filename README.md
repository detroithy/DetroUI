# Plasma Desktop

KDE Plasma ve Hyprland benzeri, Windows 10 için özelleştirilebilir masaüstü ortamı.

## Özellikler

### Widget'lar
- **Saat & Takvim** - Analog/dijital saat ve takvim görünümü
- **Sistem Monitörü** - CPU, RAM, disk ve ağ kullanımı
- **Hava Durumu** - Gerçek zamanlı hava durumu ve 5 günlük tahmin
- **Notlar** - Hızlı not alma ve düzenleme
- **Müzik Player** - Medya kontrolü

### Arayüz
- **Özel Görev Çubuğu** - KDE Plasma benzeri görev çubuğu
- **Uygulama Başlatıcı** - Arama ve kategorilere göre uygulama listesi
- **Sistem Tepsisi** - Saat, ağ, ses ve pil durumu
- **Ayarlar Paneli** - Tema ve özelleştirme seçenekleri

### Kısayollar
- `Super + Space` - Uygulama Başlatıcı
- `Super + T` - Ayarlar
- `Super + W` - Widget'ları Aç/Kapat

## Kurulum

```bash
# Bağımlılıkları yükle
npm install

# Uygulamayı başlat
npm start

# Windows için paketle
npm run build
```

## Teknolojiler

- **Electron** - Masaüstü çerçeve
- **systeminformation** - Sistem izleme
- **openmeteo** - Hava durumu API'si
- **electron-store** - Kalıcı veri saklama

## Yapılandırma

Ayarlar panelinden tema, vurgu rengi, widget ayarları ve daha fazlasını özelleştirebilirsiniz.

## Lisans

MIT
