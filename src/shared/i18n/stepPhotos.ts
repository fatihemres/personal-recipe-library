export const stepPhotosTr = {
  heading: 'Adım fotoğrafı', add: 'Fotoğraf ekle', replace: 'Fotoğrafı değiştir', remove: 'Fotoğrafı taslaktan kaldır',
  unavailable: 'Adım fotoğrafı okunamıyor. Kayıtlı bağlantı korunuyor.',
  preview: 'Fotoğrafı önizle', close: 'Önizlemeyi kapat',
  help: 'İsteğe bağlı tek fotoğraf. Yerel JPEG, PNG veya statik WebP; en fazla 20 MB / 30 megapiksel. Kaydetmeden tarife uygulanmaz.',
  deleteTitle: 'Fotoğraflı adım taslaktan kaldırılsın mı?',
  deleteBody: 'Bu adımın fotoğraf bağlantısı da taslaktan kaldırılır. Kayıtlı tarif ancak Kaydet ile değişir; geçmiş ve paylaşılan görseller silinmez.',
  deleteConfirm: 'Adımı taslaktan kaldır', cancel: 'Vazgeç',
  legacy: 'Bu eski sürüm adım fotoğrafları içermez. Geri yükleme güncel adım fotoğrafı bağlantılarını kaldırır; dosyalar ve geçmiş korunur.',
  label: (position: number) => `${position}. adım fotoğrafı`,
  previewLabel: (position: number) => `${position}. adım fotoğrafını önizle`,
} as const;
