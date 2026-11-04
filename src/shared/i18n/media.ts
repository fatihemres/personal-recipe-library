export const mediaTr = {
 privacy:'Görsel gizliliği ve saklama',retention:'Kapağı kaldırmak veya tarifi kalıcı silmek, uygulamadaki görsel kopyalarını silmez. Taslak ve geçmiş başvuruları korunur; bu aşamada otomatik disk temizliği yoktur.',
 cover:'Tarif kapağı', add:'Kapak fotoğrafı ekle',replace:'Kapak fotoğrafını değiştir',remove:'Kapağı taslaktan kaldır',
 help:'İsteğe bağlı. JPEG, PNG veya statik WebP; en fazla 20 MB ve 30 megapiksel. Özgün dosyanız değiştirilmez. Ayrı yerel kopyadan konum ve EXIF bilgileri kaldırılır. Renk profili ve yüksek bit derinliği korunmaz.',
 saveHelp:'Kapak değişikliği taslakta tutulur; tarife uygulamak için ayrıca Kaydet gerekir.',
 importing:'Görsel yerel olarak hazırlanıyor…',unavailable:'Kapak görseli okunamıyor. Kayıtlı bağlantı korunuyor.',retry:'Görseli yeniden yükle',
 legacy:'Bu eski sürüm kapak içermiyor. Geri yüklemek güncel kapağı kaldırır; dosya güvenle saklanır.',
} as const;
export const mediaErrors = {
 'errors.mediaLimit':'Görsel sınırı aşıldı: en fazla 20.000.000 dosya baytı ve 30.000.000 piksel. İşleme bellek sınırını aşan görselleri küçültüp yeniden deneyin.',
 'errors.mediaUnsupported':'Bu görsel türü desteklenmiyor. Statik JPEG, PNG veya WebP seçin. HEIC/HEIF ve hareketli görseller bu sürümde desteklenmez.',
 'errors.mediaInvalid':'Görsel bozuk veya eksik; dosya çözümlenemedi. Başka bir yerel görsel seçin.',
 'errors.mediaPath':'Seçilen dosyaya güvenli erişim sağlanamadı. Sembolik bağlantı yerine gerçek yerel dosyayı seçin.',
 'errors.mediaUnavailable':'Görsel dosyası eksik, değiştirilmiş veya okunamıyor. Kayıtlar silinmedi; dosya erişimini kontrol edip yeniden deneyin.',
 'errors.mediaBusy':'Görsel seçimi veya aktarımı sürüyor. Tamamlayın ya da dosya seçimini iptal edip yeniden deneyin.',
} as const;
