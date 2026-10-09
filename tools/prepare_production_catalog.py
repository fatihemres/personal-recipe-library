#!/usr/bin/env python3
"""Reproduce the reviewed production selection; no downloads or automatic translation."""
import csv
import hashlib
import io
import json
import pathlib
import sys
import zipfile
import unicodedata
from collections import Counter
from prepare_validation_catalog import ARCHIVE_HASH, CATEGORIES, RECORDS, encoded, stable

ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = ROOT / 'catalog/production'
# Consumer-facing groups, not a wholesale import of USDA's scientific taxonomy.
GROUPS = [
 ('vegetables','food','Sebzeler','Vegetables'), ('herbs','food','Otlar','Herbs'),
 ('spices','food','Baharatlar','Spices'), ('grains','food','Tahıllar','Grains'),
 ('legumes','food','Baklagiller','Legumes'), ('nuts-seeds','food','Kuruyemiş ve tohumlar','Nuts and seeds'),
 ('meat','food','Et','Meat'), ('poultry','food','Kümes hayvanları','Poultry'),
 ('seafood','food','Deniz ürünleri','Seafood'), ('dairy-alternatives','food','Süt alternatifleri','Dairy alternatives'),
 ('baking','food','Un ve fırıncılık','Flour and baking'), ('sauces','food','Soslar','Sauces'),
 ('condiments','food','Çeşniler','Condiments'), ('fermented','food','Fermente gıdalar','Fermented foods'),
 ('turkish-regional','food','Yöresel Türk malzemeleri','Regional Turkish ingredients'),
 ('international-specialty','food','Uluslararası özel malzemeler','International specialty ingredients'),
 ('water','beverage','Su ve maden suyu','Water and mineral water'), ('juices','beverage','Meyve ve sebze suları','Fruit and vegetable juices'),
 ('coffee','coffee-tea','Kahve','Coffee'), ('tea','coffee-tea','Çay','Tea'),
 ('infusions','beverage','Bitki demlemeleri','Herbal infusions'), ('syrups','beverage','Şuruplar','Syrups'),
 ('mixers','beverage','Karışım içecekleri','Mixers'), ('soft-drinks','beverage','Gazlı içecekler','Sodas and soft drinks'),
 ('garnishes','beverage','Süsleme malzemeleri','Garnishes'), ('spirits','beverage','Distile alkollü içkiler','Spirits'),
 ('liqueurs','beverage','Likörler','Liqueurs'), ('wines','beverage','Şaraplar','Wines'),
 ('beers','beverage','Biralar','Beers'), ('bitters','beverage','Kokteyl bitterleri','Cocktail bitters'),
 ('other-beverages','beverage','Diğer içecek malzemeleri','Other beverage ingredients'),
 ('cheeses','dairy','Peynirler','Cheeses'), ('milk-powders','dairy','Süt tozları','Milk powders'),
 ('dried-fruit','fruit','Kuru meyveler','Dried fruits'), ('mushrooms','vegetables','Mantarlar','Mushrooms'),
 ('flours','baking','Unlar','Flours'), ('noodles-pasta','grains','Makarna ve erişte','Noodles and pasta'),
 ('chocolate-cocoa','baking','Çikolata ve kakao','Chocolate and cocoa'),
 ('whisky','spirits','Viski','Whisky'), ('gin','spirits','Cin','Gin'),
 ('brandy','spirits','Brandi ve meyve distilatları','Brandy and fruit spirits'),
 ('traditional-drinks','other-beverages','Geleneksel Türk içecekleri','Traditional Turkish drinks'),
]

def save(name, value):
 data = encoded(value)
 (OUT / name).write_bytes(data)
 return {'file': name, 'sha256': hashlib.sha256(data).hexdigest()}

def main():
 raw = pathlib.Path(sys.argv[1]).read_bytes()
 if hashlib.sha256(raw).hexdigest() != ARCHIVE_HASH:
  raise ValueError('Pinned USDA archive checksum mismatch')
 rows = list(csv.DictReader((OUT / 'curation.tsv').open(encoding='utf-8'), delimiter='\t'))
 for batch in sorted((OUT / 'batches').glob('*.tsv')):
  rows.extend(csv.DictReader(batch.open(encoding='utf-8'), delimiter='\t'))
 ids = [r['fdc_id'] for r in rows]
 if len(set(ids)) != len(ids): raise ValueError('Duplicate selected USDA IDs')
 with zipfile.ZipFile(io.BytesIO(raw)) as z:
  with io.TextIOWrapper(z.open('FoodData_Central_sr_legacy_food_csv_2018-04/food.csv'), encoding='utf-8-sig') as f:
   facts = {r['fdc_id']: r['description'] for r in csv.DictReader(f) if r['fdc_id'] in ids}
 if len(facts) != len(rows): raise ValueError('Missing source identities')
 categories = {c['key']: c for c in CATEGORIES}
 categories.update({k:dict(key=k,parent=p,tr=tr,en=en) for k,p,tr,en in GROUPS})
 old = {r[0]:r for r in RECORDS}
 entries, curation = [], []
 for r in rows:
  external = r['fdc_id']
  if facts[external] != r['expected_description']: raise ValueError('Source identity/form changed: '+external)
  category = r['category']
  if category not in categories: raise ValueError('Unknown category')
  key = old[external][1] if external in old else 'usda-sr-'+external
  beverage = category in {k for k,p,_,_ in GROUPS if p in ('beverage','coffee-tea')} or category == 'coffee-tea'
  unit, dims = ('mL',['volume']) if beverage else ('g',['mass'])
  if category in ('vegetables','fruit','eggs'): dims=['mass','count']
  if category in ('oils','dairy','dairy-alternatives','sauces'): dims=['mass','volume']
  if external in old: unit,dims=old[external][7:9]
  if external in ('171893','174133','173230','173454','170877'): unit,dims='g',['mass']
  memberships = [category]
  if category == 'coffee-tea': memberships.append('coffee')
  if external in ('168155','169097','167746','173475'): memberships.append('garnishes')
  if external in ('172217','169640','168155'): memberships.append('beverage')
  if external in ('174277','174278','172886','174531','174529','167763','174120'): memberships.append('international-specialty')
  descriptor=facts[external].lower()
  for child, matched in [('cheeses',descriptor.startswith('cheese,')),('milk-powders',external in ('173454','170877')),('dried-fruit',category=='fruit' and any(x in descriptor for x in ('dried','raisins','dates,'))),('mushrooms',descriptor.startswith('mushrooms,')),('flours','flour' in descriptor and category=='baking'),('noodles-pasta',any(x in descriptor for x in ('noodles','pasta,','spaghetti','macaroni')) and category=='grains'),('chocolate-cocoa',category=='baking' and ('chocolate,' in descriptor or 'cocoa,' in descriptor))]:
   if matched: memberships.append(child)
  ta=[x for x in r['aliases_tr'].split(';') if x and x != r['tr']]
  ea=[x for x in r['aliases_en'].split(';') if x and x != r['en']]
  # Culinary labels/classification are authored separately from the unchanged USDA descriptor.
  classification=dict(tr=r['tr'],en=r['en'],aliasesTr=ta,aliasesEn=ea,categories=memberships,preferredUnit=unit,dimensions=dims)
  desc=json.dumps(classification,ensure_ascii=False,sort_keys=True,separators=(',',':'))
  curation.append(dict(externalId=key,description=desc))
  entries.append(dict(id=stable('ingredient',key),key=key,canonicalName=r['en'],ingredientType='beverage' if beverage else 'food',names=dict(tr=r['tr'],en=r['en']),aliases=[dict(locale='tr',name=a) for a in ta]+[dict(locale='en',name=a) for a in ea],categories=memberships,preferredUnit=unit,dimensions=dims,relations=[dict(key='sugar-granulated',kind='related')] if key=='sugar-powdered' else [],provenance=[dict(sourceId='usda-sr-legacy',sourceVersion='2018-04-selection-3',externalId=external,description=facts[external]),dict(sourceId='recipeatlas-curation',sourceVersion='production-2',externalId=key,description=desc)],observations=[]))
 # These original entries cite primary references for identity existence, not a
 # redistributed third-party database. No fabricated USDA IDs or observations.
 authored=json.loads((OUT/'reference-curation.json').read_text())
 for r in authored:
  category=r['category']; key=r['key']
  if category not in categories or not r['references']: raise ValueError('Unreferenced authored identity')
  beverage=category in ('spirits','liqueurs','syrups','bitters','other-beverages','garnishes')
  memberships=[category]
  if category in ('grains','baking','sweeteners','dairy'): memberships.append('turkish-regional')
  if category=='other-beverages': memberships+=['traditional-drinks','turkish-regional']
  if category=='spirits':
   if 'whisky' in key or key=='project-bourbon': memberships.append('whisky')
   if 'gin' in key: memberships.append('gin')
   if any(x in key for x in ('brandy','cognac','armagnac','calvados')): memberships.append('brandy')
  unit,dims=('mL',['volume']) if beverage else ('g',['mass'])
  if category=='garnishes': unit,dims='g',['mass','count']
  classification=dict(tr=r['tr'],en=r['en'],aliasesTr=r['aliasesTr'],aliasesEn=r['aliasesEn'],categories=memberships,preferredUnit=unit,dimensions=dims,references=r['references'],basis=r['basis'])
  desc=json.dumps(classification,ensure_ascii=False,sort_keys=True,separators=(',',':'))
  curation.append(dict(externalId=key,description=desc))
  entries.append(dict(id=stable('ingredient',key),key=key,canonicalName=r['en'],ingredientType='beverage' if beverage else 'food',names=dict(tr=r['tr'],en=r['en']),aliases=[dict(locale='tr',name=a) for a in r['aliasesTr']]+[dict(locale='en',name=a) for a in r['aliasesEn']],categories=memberships,preferredUnit=unit,dimensions=dims,relations=[],provenance=[dict(sourceId='recipeatlas-curation',sourceVersion='production-2',externalId=key,description=desc)],observations=[]))
 # Name uniqueness is locale-aware in Rust; also fail fast on exact duplicate labels here.
 for locale in ('tr','en'):
  names=[e['names'][locale] for e in entries]
  if len(names)!=len(set(names)): raise ValueError('Duplicate localized names')
 usda=save('usda-records.json',[dict(externalId=i,description=facts[i]) for i in sorted(facts)])
 curated=save('curation-records.json',sorted(curation,key=lambda x:x['externalId']))
 artifact=save('ingredients.json',dict(categories=sorted(categories.values(),key=lambda x:x['key']),ingredients=sorted(entries,key=lambda x:x['key'])))
 sources=[dict(id='usda-sr-legacy',version='2018-04-selection-3',name='USDA SR Legacy April 2018 — production selection 3',url='https://fdc.nal.usda.gov/download-datasets/',license='CC0-1.0',attribution='USDA Agricultural Research Service, FoodData Central, SR Legacy April 2018; selected descriptors unchanged. Selection version is project-owned, not a new USDA release.',retrievedAt='2026-10-08',artifactSha256=usda['sha256']),dict(id='recipeatlas-curation',version='production-2',name='Tarif Atlası production label curation',url='https://github.com/fatihemres/personal-recipe-library',license='CC0-1.0',attribution='Project-authored Turkish/English culinary labels, aliases and classifications; not USDA translations. No empirical metadata inferred.',retrievedAt='2026-10-09',artifactSha256=curated['sha256'])]
 save('manifest.json',dict(formatVersion=1,dataset='recipeatlas-ingredients',version=3,purpose='production',artifact=artifact,evidence={'usda-sr-legacy':usda,'recipeatlas-curation':curated},sources=sources))
 counts=Counter(c for e in entries for c in e['categories'])
 name_owners={}
 for e in entries:
  for locale,name in list(e['names'].items())+[(a['locale'],a['name']) for a in e['aliases']]:
   normalized=unicodedata.normalize('NFC',name)
   if locale == 'tr': normalized=normalized.replace('I','ı').replace('İ','i')
   normalized=' '.join(normalized.lower().split())
   name_owners.setdefault((locale,normalized),set()).add(e['key'])
 duplicate_candidates=[dict(locale=locale,name=name,keys=sorted(keys)) for (locale,name),keys in sorted(name_owners.items()) if len(keys)>1]
 save('coverage.json',dict(dataset='recipeatlas-ingredients',version=3,selectedRecords=len(entries),usdaRecords=len(rows),projectOnlyRecords=len(authored),ingredientTypeCounts=dict(Counter(e['ingredientType'] for e in entries)),aliasCount=sum(len(e['aliases']) for e in entries),duplicateNameCandidates=duplicate_candidates,productionReadyRecords=len(entries),productionReadyMeaning="Validated identity/provenance and importer compatibility; not independent linguistic certification",linguisticReviewCount=len(json.loads((OUT/'localization-review.json').read_text())),importedRecords=None,importedRecordsExplanation='Package coverage; actual database import results are separate CLI/runtime reports.',rejectedRecords=0,duplicateSelectedIds=0,sharedValidationIdentities=len(old),previousIngredientCount=235,newlyAddedIngredients=len(entries)-235,additionalIdentities=len(entries)-len(old),incompleteTranslations=0,localizationReview='Project semantic review against pinned English descriptors; no independent Turkish culinary expert review.',missingMetadata={k:len(entries) for k in ('nutrition','density','allergens','abv')},sourceCounts={'usda-sr-legacy':len(rows),'recipeatlas-curation':len(entries)},categoryMembershipCounts=dict(sorted(counts.items())),categoryNodes=len(categories),emptyCategories=sorted(set(categories)-set(counts)-{'food','beverage','coffee-tea'})))
 print(f'Prepared {len(entries)} production identities ({len(entries)-235} added since v2), {len(categories)} categories.')
if __name__ == '__main__': main()
