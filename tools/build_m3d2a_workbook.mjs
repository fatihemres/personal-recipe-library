// Run from repository root using the bundled artifact-tool runtime. No production writes.
import fs from 'node:fs/promises';
import path from 'node:path';
import process from 'node:process';
import console from 'node:console';
import { Workbook, SpreadsheetFile } from '@oai/artifact-tool';
const root = process.cwd();
const folder = path.join(root, 'catalog/preparation/m3d2a');
const read = async name => JSON.parse(await fs.readFile(path.join(folder, name), 'utf8'));
const [candidates, prior, sources, coverage] = await Promise.all(['candidates.json', 'prior-review.json', 'sources.json', 'coverage.json'].map(read));
if (candidates.some(r => r.importAuthorized || r.humanDecision) || prior.length !== 7 || prior.some(r => r.humanDecision)) throw new Error('Unexpected approval state');
const wb = Workbook.create();
const status = {
  'proposed-after-editorial-review': 'Editoryal kontrol sonrası öneri',
  'defer-source-clearance': 'Kaynak izni incelemesi bekliyor',
  'review-identity-or-terminology': 'Kimlik / terim incelemesi gerekli',
  'defer-probable-duplicate': 'Olası mevcut kimlik — beklet',
  'reject-brand-specific': 'Markaya özgü — öneriden çıkar',
};
const headers = ['Aday kimliği', 'Önerilen Türkçe ad', 'İngilizce kaynak adı', 'Öneri durumu', 'İnsan kararı', 'İnceleyen / tarih', 'Terminoloji güveni', 'Önerilen kategoriler', 'Tür', 'Form / hazırlama', 'Önerilen takma adlar', 'Çakışma / inceleme konusu', 'Kaynak kimliği', 'Kaynak bağlantısı', 'Lisans durumu'];
const matrix = rows => rows.map(r => [r.candidateId, r.proposedTurkishName, r.proposedEnglishName, status[r.proposedImportStatus], '', '', r.terminologyConfidence.startsWith('high') ? 'Yüksek editoryal; insan onayı yok' : 'Orta editoryal; insan incelemesi gerekli', r.proposedCategories.join(', '), r.ingredientType === 'food' ? 'Gıda' : 'İçecek', r.retainedSourceForm, r.proposedAliases.join('; ') || 'Önerilmedi', [r.concern, r.editorialCollisionAssessment?.reason, ...r.collisions.map(c => `${c.key}: ${c.tr} (${c.kind})`)].filter(Boolean).join('\n') || 'Otomatik eşleşme yok; anlamsal inceleme yine gerekli', `${r.source}:${r.sourceIdentifier}`, r.sourceUrl, r.licenseStatus]);
const ranges = [];
function table(name, heading, note, cols, rows, widths, decisions = false) {
  const s = wb.worksheets.add(name); s.showGridLines = false;
  s.getRange('A2').values = [[heading]]; s.getRange('A2').format.font = { name: 'Arial', size: 15, bold: true, color: '#354C40' };
  s.getRange('A3').values = [[note]]; s.getRange('A3').format.font = { name: 'Arial', size: 10, italic: true, color: '#536158' };
  const end = String.fromCharCode(64 + cols.length); const last = rows.length + 5;
  s.getRange(`A5:${end}${last}`).values = [cols, ...rows];
  const all = s.getRange(`A5:${end}${last}`); all.format.font = { name: 'Arial', size: 11, color: '#25352B' }; all.format.wrapText = true; all.format.verticalAlignment = 'center';
  s.getRange(`A5:${end}5`).format.fill = '#354C40'; s.getRange(`A5:${end}5`).format.font = { name: 'Arial', size: 11, bold: true, color: '#FFFFFF' }; s.getRange(`A5:${end}5`).format.rowHeight = 36;
  rows.forEach((row, index) => {
    const lines = Math.max(...row.map((value, col) => String(value).split('\n').reduce((n, part) => n + Math.max(1, Math.ceil(part.length / (widths[col] * 0.85))), 0)));
    s.getRangeByIndexes(index + 5, 0, 1, cols.length).format.rowHeight = Math.max(32, lines * 15 + 12);
  });
  widths.forEach((w,i) => { s.getRangeByIndexes(4,i,rows.length+1,1).format.columnWidth = w; });
  const t = s.tables.add(`A5:${end}${last}`, true, `Review${ranges.length}`); t.showFilterButton = true; t.showBandedRows = true;
  s.freezePanes.freezeRows(5);
  if (decisions) {
    s.getRange(`E6:F${last}`).format.fill = '#FFF1CE';
    s.getRange(`E6:E${last}`).dataValidation = { rule: { type: 'list', values: ['Kabul önerisi', 'Düzeltme gerekli', 'Reddet', 'Beklet'] } };
  }
  ranges.push([name, `A2:D${Math.min(last,9)}`]);
  return s;
}
const high = candidates.filter(r => r.proposedImportStatus === 'proposed-after-editorial-review');
const unresolved = matrix(candidates.filter(r => r.proposedImportStatus !== 'proposed-after-editorial-review'));
for (const p of prior) unresolved.push([p.key, p.tr, p.en, 'Önceki karar bekliyor — değiştirilmedi', '', '', 'İnsan incelemesi gerekli', p.categories.join(', '), 'Önceki kayıt', p.descriptor || p.en, 'Değiştirilmedi', p.reason, p.key, p.sourceUrl, 'Önceki kaynak; karar verilmedi']);
const widths = [31,39,49,32,26,26,31,32,17,52,26,77,30,60,57];
table('Yüksek güvenli adaylar', `${high.length} öncelikli aday — M3D-2A`, 'Sarı hücreler insan kararları içindir. Editoryal güven, bağımsız uzman onayı veya içe aktarma izni değildir.', headers, matrix(high), widths, true);
table('İnceleme bekleyenler', `${unresolved.length} inceleme satırı`, '128 yeni aday + 7 önceki açık karar. Ret / beklet durumu üretime ekleme izni vermez.', headers, unresolved, widths, true);
table('Kategori kapsamı', 'Release 7 — gerçek SQLite kapsamı', '568 malzeme / 269 takma ad / 50 kategori. Kategori üyelikleri örtüşür; toplanarak toplam bulunmaz.', ['Kategori anahtarı', 'Türkçe kategori', 'Üst kategori', 'Doğrudan kayıt', 'Altlarla tekil kayıt', 'Tüm adaylar', 'Öncelikli adaylar'], coverage.categories.map(r => [r.key,r.tr,r.parent || '—',r.direct,r.includingDescendants,r.candidates,r.higherConfidenceCandidates]), [29,36,27,22,25,22,24]);
table('Kaynak lisansları', 'Kaynak değerlendirmesi — 14 Ekim 2026', 'Adaylar üretim kataloğundan ayrıdır; sıfır kayıt içe aktarıldı. Ayrıntılı kararlar kaynak manifestindedir.', ['Kaynak', 'Lisans', 'Üretim kararı', 'Ticari kullanım', 'Yükümlülükler', 'Diller', 'Kimlikler / format', 'Sürüm', 'Resmî bağlantı', 'Sınırlamalar'], sources.map(s => [s.name,s.license,s.decision,s.commercial,s.obligations,s.languages,`${s.identifiers}; ${s.formats}`,s.version,s.url,s.limitations]), [29,41,51,38,85,37,45,55,68,68]);
wb.recalculate();
console.log((await wb.inspect({kind:'table',range:"'Yüksek güvenli adaylar'!A5:F8",include:'values',tableMaxRows:4,tableMaxCols:6})).ndjson);
console.log((await wb.inspect({kind:'match',searchTerm:'#REF!|#DIV/0!|#VALUE!|#NAME\\?',options:{useRegex:true,maxResults:10},summary:'Formula errors'})).ndjson);
for (const [sheetName, range] of ranges) {
  const png = await wb.render({sheetName,range,scale:1,format:'png'});
  await fs.writeFile(`/private/tmp/m3d2a-${ranges.findIndex(r => r[0]===sheetName)}.png`,new Uint8Array(await png.arrayBuffer()));
}
for (const [label, sheetName, range] of [['decisions','Yüksek güvenli adaylar','E5:H9'], ['evidence','İnceleme bekleyenler','L5:O8'], ['prior','İnceleme bekleyenler','A134:D140'], ['rights','Kaynak lisansları','E5:J8']]) {
  const png = await wb.render({sheetName,range,scale:1,format:'png'});
  await fs.writeFile(`/private/tmp/m3d2a-${label}.png`,new Uint8Array(await png.arrayBuffer()));
}
const output = await SpreadsheetFile.exportXlsx(wb);
await output.save(path.join(root,'docs/INGREDIENT_M3D2A_CANDIDATE_REVIEW.xlsx'));
