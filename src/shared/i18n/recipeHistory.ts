export const historyTr={
 heading:'Sürüm geçmişi',intro:'Kaydedilmiş tarif içeriği. Özel Markdown notları, favoriler ve koleksiyonlar bu geçmişe dahil değildir.',
 open:'Geçmişi aç',close:'Geçmişi kapat',loading:'Sürüm geçmişi okunuyor…',retry:'Yeniden dene',empty:'Bu tarif için saklanan sürüm yok.',more:'Daha eski sürümler',
 created:'İlk kayıt',edited:'İçerik düzenlemesi',upgrade_baseline:'Yükseltme başlangıç kaydı',restored:'Geçmişten geri yükleme',
 baselineHelp:'Bu kayıt, uygulama yükseltilirken mevcut içeriği yakalar; tarifin ilk oluşturulduğu hâli veya önceki düzenlemeler değildir.',
 sourceRevision:'Kaynak kayıt revizyonu',restoredFrom:'Geri yüklenen kaynak',preview:'Sürümü görüntüle',selected:'Seçili sürüm',compare:'Karşılaştır',compareWith:'Karşılaştırma hedefi',current:'Güncel kaydedilmiş içerik',
 before:'Seçili geçmiş sürüm',after:'Karşılaştırma hedefi',field:'Alan',noChanges:'Karşılaştırılan içerikte fark yok.',moreChanges:'Diğer farkları göster',
 restore:'Bu sürümü geri yükle',confirmTitle:'Kaydedilmiş tarif içeriği değiştirilsin mi?',
 confirmBody:'Seçilen sürümün içeriği, aynı tarif kimliğinde güncel içeriğin yerini alır ve yeni bir sürüm olarak kaydedilir. Önceki sürümler, favoriler ve özel Markdown notları korunur. Açık taslak varsa işlem durdurulur.',
 cancel:'Vazgeç',restoredNotice:'İçerik yeni bir sürüm olarak geri yüklendi.',unchanged:'Seçilen içerik zaten güncel. Yeni sürüm oluşturulmadı.',inactive:'Geçmiş salt okunur. İçeriği geri yüklemek için önce tarifi arşivden veya çöp kutusundan çıkarın.',
 identities:'Saklanan satır ve adım kimlikleri',line:'Malzeme satırı',step:'Adım',missing:'Yok',
 fields:{formatVersion:'Biçim',recipeId:'Tarif kimliği',title:'Başlık',description:'Açıklama',kind:'Tarif türü',servings:'Porsiyon',prepMinutes:'Hazırlık süresi',cookMinutes:'Pişirme süresi',notes:'Tarif notu',ingredients:'Malzeme satırları (sıralı)',ingredientNames:'Yakalanan malzeme adları',steps:'Adımlar (sıralı)',id:'Satır / adım kimliği',ingredientId:'Malzeme kimliği',quantity:'Miktar metni',unitCode:'Birim',note:'Satır notu',instructions:'Talimat',details:'İleri adım bilgileri',durationMinutes:'Süre (dakika)',temperatureValue:'Sıcaklık',temperatureUnit:'Sıcaklık birimi',heatLevel:'Isı düzeyi',equipment:'Ekipman',technique:'Teknik',tips:'İpuçları',ingredientLineIds:'Bağlı malzeme satırları'} as Record<string,string>,
} as const;
