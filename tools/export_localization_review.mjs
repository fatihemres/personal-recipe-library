// Run with the bundled @oai/artifact-tool runtime. CSV output needs no runtime dependency.
import fs from 'node:fs/promises';
import process from 'node:process';
import console from 'node:console';
import path from 'node:path';
import { Workbook } from '@oai/artifact-tool';

const root = process.argv[2];
if (!root) throw new Error('Pass repository root');
const queue = JSON.parse(await fs.readFile(path.join(root, 'catalog/production/localization-review.json'), 'utf8'));
const headings = ['stable_id', 'canonical_key', 'existing_turkish', 'existing_english', 'categories', 'source_ids', 'source_versions', 'external_ids', 'references', 'linguistic_concern', 'proposed_turkish', 'confidence', 'confidence_reason', 'human_review_required', 'human_approved', 'decision'];
const rows = queue.map(q => [q.id, q.key, q.existingTr, q.existingEn, q.categories.join('; '), q.sourceReferences.map(s => s.sourceId).join('; '), q.sourceReferences.map(s => s.version).join('; '), q.sourceReferences.map(s => s.externalId).join('; '), [...new Set(q.sourceReferences.flatMap(s => s.primaryReferences?.length ? s.primaryReferences : [s.url]))].join('; '), q.reason, q.proposedTr, q.confidence, q.confidenceReason, String(q.humanReviewNecessary), String(q.humanApproved), q.decision]);
const wb = Workbook.create();
const sheet = wb.worksheets.add('Terminology review');
const range = sheet.getRange(`A1:P${rows.length + 1}`);
range.values = [headings, ...rows];
range.format.font = {name: 'Arial', size: 10};
range.format.columnWidth = 34;
range.format.wrapText = true;
range.format.rowHeight = 70;
sheet.getRange('A1:P1').format = {fill: '#34463B', font: {bold: true, color: '#FFFFFF'}, rowHeight: 32};
sheet.showGridLines = false;
sheet.freezePanes.freezeRows(1);
wb.recalculate();
const check = await wb.inspect({kind: 'table', range: 'Terminology review!A1:D4', include: 'values', tableMaxRows: 4, tableMaxCols: 4, maxChars: 1500});
console.log(check.ndjson);
const values = range.values;
if (values.length !== rows.length + 1 || values.some(row => row.length !== 16)) throw new Error('Unexpected review shape');
// CSV preserves full text; rendering is a QA preview only, not an extra deliverable.
const preview = await wb.render({sheetName: sheet.name, range: 'A1:D6', scale: 1, format: 'png'});
await fs.writeFile('/private/tmp/m3b3-review-preview.png', new Uint8Array(await preview.arrayBuffer()));
const quote = value => '"' + String(value ?? '').replaceAll('"', '""') + '"';
await fs.writeFile(path.join(root, 'docs/INGREDIENT_LOCALIZATION_REVIEW.csv'), values.map(row => row.map(quote).join(',')).join('\n') + '\n', 'utf8');
const md = ['# Ingredient terminology review — M3B-3', '', 'All 17 original items and one new endive terminology question remain explicitly pending human review. Proposals retain existing labels when stronger Turkish evidence is absent. Confidence is project judgment about identity/form matching, not independent approval.', ''];
for (const q of queue) {
  md.push(`## ${q.key}`, '', `- Stable ID: ${q.id}`, `- Turkish: ${q.existingTr}`, `- English: ${q.existingEn}`, `- Categories: ${q.categories.join(', ')}`, `- Concern: ${q.reason}`, `- Proposal: ${q.proposedTr}`, `- Confidence: ${q.confidence}. ${q.confidenceReason}`, `- Human review necessary: ${q.humanReviewNecessary}; human approved: ${q.humanApproved}.`, `- Decision: ${q.decision}`, '- Provenance:');
  for (const s of q.sourceReferences) md.push(`  - ${s.sourceId} / ${s.version} / ${s.externalId}. ${s.primaryReferences?.length ? s.primaryReferences.join(' ') : s.url}`);
  md.push('');
}
await fs.writeFile(path.join(root, 'docs/INGREDIENT_TERMINOLOGY_REVIEW.md'), md.join('\n').trimEnd() + '\n');
console.log(`Exported ${rows.length} complete UTF-8 review rows; human approvals: 0`);
