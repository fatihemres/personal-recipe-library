// Run with the bundled @oai/artifact-tool runtime from a temporary directory.
// Source rows remain in docs/INGREDIENT_LOCALIZATION_REVIEW.csv.
import fs from 'node:fs/promises';
import path from 'node:path';
import process from 'node:process';
import { Workbook, SpreadsheetFile } from '@oai/artifact-tool';

const root = process.cwd();
const source = await fs.readFile(path.join(root, 'docs/INGREDIENT_LOCALIZATION_REVIEW.csv'), 'utf8');
const parsed = await Workbook.fromCSV(source, { sheetName: 'Source' });
const rows = parsed.worksheets.getItemAt(0).getUsedRange().values;
const [header, ...body] = rows;
const index = Object.fromEntries(header.map((name, position) => [name, position]));
// Preserve the final 18-item historical review independently of later expansion.
const catalog = JSON.parse(await fs.readFile(path.join(root, 'catalog/releases/6/ingredients.json'), 'utf8'));
const byId = new Map(catalog.ingredients.map(record => [record.id, record]));
if (body.length !== 18 || byId.size !== 508) throw new Error('Unexpected source review or catalog size');
const workbook = Workbook.create();
const sheet = workbook.worksheets.add('Terim incelemesi');
sheet.showGridLines = false;
sheet.tabColor = '#8A6840';
const columns = ['Malzeme ID', 'Sabit anahtar', 'Önceki Türkçe ad', 'Önceki İngilizce ad', 'Diğer adlar', 'Kategoriler', 'İnceleme nedeni', 'Öneri', 'Kanıt bağlantıları', 'Güven ve gerekçe', 'Durum', 'Karar kodu', 'Seçilen Türkçe ad', 'Seçilen İngilizce ad', 'İnceleyen / tarih'];
const clean = value => String(value ?? '').replaceAll('\r', ' ').replaceAll('\n', ' ');
const matrix = body.map(row => {
  const get = name => clean(row[index[name]]);
  const record = byId.get(get('stable_id'));
  if (!record) throw new Error(`Missing catalog identity ${get('stable_id')}`);
  const aliases = record.aliases.map(alias => `${alias.locale}: ${alias.name}`).join(' · ');
  const decision = get('decision');
  const status = decision === 'user-approved-name' ? 'Kullanıcı adı uygulandı' : decision === 'user-authorized-retain-name' ? 'Ad korundu (kullanıcı kararı)' : 'Ek doğrulama gerekli';
  return [get('stable_id'), get('canonical_key'), get('existing_turkish'), get('existing_english'), aliases, get('categories'), get('linguistic_concern'), get('proposed_turkish'), get('references'), `${get('confidence')}: ${get('confidence_reason')}`, status, decision, get('selected_turkish'), get('selected_english'), ''];
});
sheet.getRange('A1:O19').values = [columns, ...matrix];
sheet.getRange('A1:O1').format.fill = '#253E34';
sheet.getRange('A1:O1').format.font = { name: 'Aptos', size: 11, bold: true, color: '#FFFFFF' };
sheet.getRange('A2:O19').format.font = { name: 'Aptos', size: 10, color: '#24332B' };
sheet.getRange('A1:O19').format.verticalAlignment = 'center';
sheet.getRange('A1:O19').format.wrapText = true;
sheet.getRange('A1:O1').format.rowHeight = 36;
sheet.getRange('A2:O19').format.rowHeight = 80;
const widths = [39, 29, 32, 31, 35, 24, 65, 30, 70, 54, 27, 34, 36, 36, 26];
for (let col = 0; col < widths.length; col++) sheet.getRangeByIndexes(0, col, 19, 1).format.columnWidth = widths[col];
sheet.getRange('M2:O19').format.fill = '#FFF3D9';
sheet.getRange('A1:O19').format.borders = { preset: 'inside', style: 'thin', color: '#D6DED8' };
const table = sheet.tables.add('A1:O19', true, 'TerminologyReview');
table.showFilterButton = true;
table.showBandedRows = true;
sheet.freezePanes.freezeRows(1);
workbook.recalculate();
const summary = await workbook.inspect({kind:'region',sheetId:sheet.sheetId,range:'A1:D4',maxChars:1600});
process.stdout.write(`${summary.ndjson}\n`);
const preview = await workbook.render({sheetName:'Terim incelemesi',range:'K12:O19',scale:1,format:'png'});
await fs.writeFile('/private/tmp/m3c5-review-preview.png', new Uint8Array(await preview.arrayBuffer()));
const output = await SpreadsheetFile.exportXlsx(workbook);
await output.save(path.join(root, 'docs/INGREDIENT_TERMINOLOGY_REVIEW.xlsx'));
await fs.rm(path.join(root, 'docs/INGREDIENT_TERMINOLOGY_REVIEW.xlsx.inspect.ndjson'), { force: true });
