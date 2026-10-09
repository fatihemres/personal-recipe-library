#!/usr/bin/env python3
"""Prepare only eight verified generic-food identities from the pinned official CSV archive.
No network access, branded database processing, nutrition inference, or production catalog claim.
"""
import csv
import hashlib
import io
import json
import pathlib
import sys
import uuid
import zipfile

ARCHIVE_HASH = 'b80817294b8850530aaedf2e515c02593b1824f763a0ff356e5c2081643e6fd0'
ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = ROOT / 'catalog' / 'validation'
# ID, canonical culinary form, tr, en, tr aliases, en aliases, category, unit, dimensions.
RECORDS = [
 ('169655','sugar-granulated','Toz şeker','Granulated sugar',['Şeker'],['White sugar'],'sweeteners','g',['mass']),
 ('169656','sugar-powdered','Pudra şekeri','Powdered sugar',['Pudra şeker'],['Icing sugar','Confectioners sugar'],'sweeteners','g',['mass']),
 ('169640','honey','Bal','Honey',[],[],'sweeteners','g',['mass','volume']),
 ('172217','milk-whole-unfortified','Tam yağlı süt (%3,25 yağ)','Whole milk (3.25%, unfortified)',['Tam yağlı süt'],['Whole milk'],'dairy','mL',['mass','volume']),
 ('171890','coffee-brewed','Demlenmiş kahve','Brewed coffee',['Demleme kahve'],['Prepared coffee'],'coffee-tea','mL',['mass','volume']),
 ('168155','lime-raw','Çiğ misket limonu','Raw lime',['Misket limonu'],['Lime'],'fruit','adet',['mass','count']),
 ('171413','olive-oil','Zeytinyağı','Olive oil',['Zeytin yağı'],[],'oils','mL',['mass','volume']),
 ('171287','egg-whole-raw','Çiğ bütün yumurta','Raw whole egg',['Yumurta'],['Whole egg'],'eggs','adet',['mass','count']),
]
CATEGORIES = [
 {'key':'food','parent':None,'tr':'Gıda malzemeleri','en':'Food ingredients'},
 {'key':'beverage','parent':None,'tr':'İçecek malzemeleri','en':'Beverage ingredients'},
 *[{'key':k,'parent':p,'tr':tr,'en':en} for k,p,tr,en in [
 ('sweeteners','food','Tatlandırıcılar','Sweeteners'),('dairy','food','Süt ürünleri','Dairy'),
 ('coffee-tea','beverage','Kahve ve çay','Coffee and tea'),('fruit','food','Meyveler','Fruit'),
 ('oils','food','Yağlar','Oils'),('eggs','food','Yumurtalar','Eggs')]],
]
def encoded(value):
 return (json.dumps(value,ensure_ascii=False,sort_keys=True,indent=2)+'\n').encode()
def save(name,value):
 data=encoded(value);(OUT/name).write_bytes(data)
 return {'file':name,'sha256':hashlib.sha256(data).hexdigest()}
def stable(kind,key):
 b=bytearray(hashlib.sha256(f'recipeatlas:{kind}:{key}'.encode()).digest()[:16]);b[6]=(b[6]&15)|128;b[8]=(b[8]&63)|128
 return str(uuid.UUID(bytes=bytes(b)))
def main():
 archive=pathlib.Path(sys.argv[1]);raw=archive.read_bytes()
 if hashlib.sha256(raw).hexdigest()!=ARCHIVE_HASH: raise ValueError('Pinned USDA archive checksum mismatch')
 with zipfile.ZipFile(io.BytesIO(raw)) as z:
  with io.TextIOWrapper(z.open('FoodData_Central_sr_legacy_food_csv_2018-04/food.csv'),encoding='utf-8-sig') as f:
   wanted={r[0] for r in RECORDS};facts={}
   for row in csv.DictReader(f):
    if row['fdc_id'] in wanted:
     if row['fdc_id'] in facts:raise ValueError('Duplicate verified USDA ID')
     facts[row['fdc_id']]=row['description']
 if len(facts)!=len(RECORDS):raise ValueError('Missing verified USDA IDs')
 OUT.mkdir(parents=True,exist_ok=True)
 usda=save('usda-records.json',[{'externalId':i,'description':facts[i]} for i in sorted(facts)])
 curation=[];entries=[]
 for external,key,tr,en,ta,ea,category,unit,dims in RECORDS:
  classification={'tr':tr,'en':en,'aliasesTr':ta,'aliasesEn':ea,'category':category,'preferredUnit':unit,'dimensions':dims}
  desc=json.dumps(classification,ensure_ascii=False,sort_keys=True,separators=(',',':'))
  curation.append({'externalId':key,'description':desc})
  entries.append({'id':stable('ingredient',key),'key':key,'canonicalName':en,
   'ingredientType':'beverage' if category=='coffee-tea' else 'food','names':{'tr':tr,'en':en},
   'aliases':[{'locale':'tr','name':a} for a in ta]+[{'locale':'en','name':a} for a in ea],
   'categories':[category]+(['beverage'] if key in ['milk-whole-unfortified','lime-raw','honey'] else []),
   'preferredUnit':unit,'dimensions':dims,
   'relations':[{'key':'sugar-granulated','kind':'related'}] if key=='sugar-powdered' else [],
   'provenance':[{'sourceId':'usda-sr-legacy','sourceVersion':'2018-04','externalId':external,'description':facts[external]},
                 {'sourceId':'recipeatlas-curation','sourceVersion':'validation-1','externalId':key,'description':desc}],
   'observations':[]})
 c=save('curation-records.json',sorted(curation,key=lambda x:x['externalId']))
 artifact=save('ingredients.json',{'categories':sorted(CATEGORIES,key=lambda x:x['key']),'ingredients':sorted(entries,key=lambda x:x['key'])})
 source_info=[('usda-sr-legacy','2018-04','USDA FoodData Central SR Legacy','https://fdc.nal.usda.gov/download-datasets/','USDA Agricultural Research Service, FoodData Central, SR Legacy April 2018; descriptions selected unchanged.',usda),
 ('recipeatlas-curation','validation-1','Tarif Atlası validation curation','https://github.com/fatihemres/personal-recipe-library','Project-authored factual Turkish/English labels, aliases and culinary classification; not USDA translations or empirical measurements.',c)]
 save('manifest.json',{'formatVersion':1,'dataset':'recipeatlas-ingredients','version':1,'purpose':'validation','artifact':artifact,'evidence':{'usda-sr-legacy':usda,'recipeatlas-curation':c},
  'sources':[{'id':i,'version':v,'name':n,'url':u,'license':'CC0-1.0','attribution':a,'retrievedAt':'2026-10-08','artifactSha256':f['sha256']} for i,v,n,u,a,f in source_info]})
 print(f'Prepared {len(entries)} validation ingredients, {len(CATEGORIES)} category nodes. Production ingredients: 0.')
if __name__=='__main__':main()
