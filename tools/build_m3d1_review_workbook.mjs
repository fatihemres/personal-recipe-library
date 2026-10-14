// Run with the bundled artifact-tool runtime, from the repository root.
import fs from 'node:fs/promises';
import path from 'node:path';
import process from 'node:process';
import console from 'node:console';
import { Workbook, SpreadsheetFile } from '@oai/artifact-tool';
const root = process.cwd();
const rows = JSON.parse(await fs.readFile(path.join(root, 'catalog/production/m3d1-review.json'), 'utf8'));
if (rows.length !== 4 || rows.some(row => row.humanDecision !== null)) throw new Error('Unexpected review status');
const workbook = Workbook.create();
const sheet = workbook.worksheets.add('M3D-1 inceleme');
sheet.showGridLines = false;
const headers = ['Sabit anahtar', 'Türkçe ad', 'İngilizce ad', 'Katalogda', 'İnceleme sorusu', 'Kaynak tanımı', 'Kaynak bağlantısı', 'Durum', 'İnsan kararı', 'İnceleyen / tarih', 'Aday UUID'];
const concerns = {
  'usda-sr-169406': 'Kavrulmamış susam içinden tahin, mevcut çiğ taşta öğütülmüş tahinle örtüşebilir. Ayrı USDA kaydı, ayrı mutfak malzemesi olduğunu kanıtlamaz. Bu aday eklenmedi.',
  'usda-sr-170007': 'Welsh onion için doğal Türkçe ad ve genel yeşil soğandan ayrımı incelenmeli. Eşdeğerlik iddiası taşıyan takma ad eklenmedi.',
  'usda-sr-168472': 'Acorn squash için Türkçe çeşit/tür adının doğal kullanımı incelenmeli. Kaynaktaki kış kabağı kimliği korunuyor.',
  'usda-sr-174208': 'Northern lobster için tercih edilen Türkçe ortak ad incelenmeli. Kaynakta bulunmayan tür eşanlamı eklenmedi.',
};
const matrix = rows.map(row => [row.key, row.tr, row.en, row.includedInRelease ? 'Evet' : 'Hayır — bekletildi', concerns[row.key], row.descriptor, row.sourceUrl, 'İnsan incelemesi bekliyor', '', '', row.id]);
sheet.getRange('A1:K5').values = [headers, ...matrix];
sheet.getRange('A1:K5').format.font = { name: 'Arial', size: 11, color: '#25352B' };
sheet.getRange('A1:K5').format.wrapText = true;
sheet.getRange('A1:K5').format.verticalAlignment = 'center';
sheet.getRange('A1:K1').format.fill = '#354C40';
sheet.getRange('A1:K1').format.font = { name: 'Arial', size: 11, color: '#FFFFFF', bold: true };
sheet.getRange('A1:K1').format.rowHeight = 32;
sheet.getRange('A2:K5').format.rowHeight = 110;
const widths = [26, 33, 33, 24, 66, 55, 58, 28, 38, 27, 39];
widths.forEach((width, i) => { sheet.getRangeByIndexes(0, i, 5, 1).format.columnWidth = width; });
sheet.getRange('I2:J5').format.fill = '#FFF1CE';
const table = sheet.tables.add('A1:K5', true, 'M3D1Review');
table.showFilterButton = true;
table.showBandedRows = true;
sheet.freezePanes.freezeRows(1);
workbook.recalculate();
console.log((await workbook.inspect({ kind: 'table', range: "'M3D-1 inceleme'!A1:D5", include: 'values', tableMaxRows: 5, tableMaxCols: 4 })).ndjson);
for (const [label, range] of [['names', 'A1:E5'], ['decisions', 'F1:K5']]) {
  const preview = await workbook.render({ sheetName: 'M3D-1 inceleme', range, scale: 1, format: 'png' });
  await fs.writeFile(`/private/tmp/m3d1-review-${label}.png`, new Uint8Array(await preview.arrayBuffer()));
}
const file = await SpreadsheetFile.exportXlsx(workbook);
await file.save(path.join(root, 'docs/INGREDIENT_M3D1_REVIEW.xlsx'));
