import { expect, test } from '@playwright/test';

test('actual editor step photos retain UUID ownership, confirm deletion and fit all desktop widths/themes',async({page})=>{
 for(const theme of ['light','dark']){
  await page.goto(`/tests/ui/workspace.html?steps=1&stepPhotos=2&theme=${theme}`);
  await page.locator('.recipe-card-open').click();
  const detail=page.locator('.detail-list');await expect(detail.locator('.step-photo-preview img')).toHaveCount(2);
  await page.getByRole('button',{name:'Düzenle',exact:true}).click();
  const rows=page.locator('[data-step-id]');const firstId=await rows.first().getAttribute('data-step-id');
  await expect(page.locator('.step-photo-editor')).toHaveCount(0);
  await rows.first().locator('.step-advanced summary').click();await rows.nth(1).locator('.step-advanced summary').click();
  await expect(page.getByRole('region',{name:'1. adım fotoğrafı',exact:true})).toBeVisible();
  for(const width of [520,640,768,1024,1440,1920]){
   await page.setViewportSize({width,height:width===520?600:900});
   expect(await page.locator('html').evaluate(e=>e.scrollWidth<=e.clientWidth+1)).toBe(true);
   for(const panel of await page.locator('.step-photo-editor').all()){
    expect(await panel.evaluate(e=>{const p=e.getBoundingClientRect();return [...e.querySelectorAll('button,img')].every(c=>{const r=c.getBoundingClientRect();return r.left>=p.left-1&&r.right<=p.right+1;});})).toBe(true);
    const actions=await panel.locator('button').evaluateAll(nodes=>nodes.map(n=>{const r=n.getBoundingClientRect();return {x:r.x,y:r.y,right:r.right,bottom:r.bottom};}));
    for(let i=0;i<actions.length;i++)for(let j=i+1;j<actions.length;j++)expect(actions[i].right<=actions[j].x+1||actions[j].right<=actions[i].x+1||actions[i].bottom<=actions[j].y+1||actions[j].bottom<=actions[i].y+1).toBe(true);
   }
   if([520,1440].includes(width))await page.getByRole('region',{name:'1. adım fotoğrafı',exact:true}).screenshot({path:test.info().outputPath(`step-photo-${theme}-${width}.png`)});
  }
  await page.setViewportSize({width:520,height:600});
  const preview=page.getByRole('button',{name:'1. adım fotoğrafını önizle',exact:true});await preview.focus();await page.keyboard.press('Enter');const dialog=page.getByRole('dialog');await expect(dialog.getByRole('button',{name:'Önizlemeyi kapat'})).toBeFocused();await expect(dialog.locator('img')).toHaveAttribute('src',/variant=preview/);await dialog.screenshot({path:test.info().outputPath(`step-photo-preview-${theme}-520.png`)});await page.keyboard.press('Escape');await expect(dialog).not.toBeVisible();await expect(preview).toBeFocused();
  await rows.first().getByRole('button',{name:'Kaldır',exact:true}).click();const removal=page.getByRole('dialog');await expect(removal).toHaveText(/Fotoğraflı adım taslaktan kaldırılsın mı/);await expect(removal.getByRole('button',{name:'Vazgeç',exact:true})).toBeFocused();await page.keyboard.press('Escape');await expect(removal).not.toBeVisible();await expect(rows).toHaveCount(2);
  const before=await page.locator(`[data-step-id="${firstId}"] img`).getAttribute('src');await page.locator(`[data-step-id="${firstId}"]`).getByRole('button',{name:/Aşağı/}).click();await expect(rows.nth(1)).toHaveAttribute('data-step-id',firstId!);await expect(rows.nth(1).locator('img')).toHaveAttribute('src',before!);
  await rows.nth(1).getByRole('button',{name:'Fotoğrafı taslaktan kaldır'}).click();await rows.nth(1).getByRole('button',{name:'Fotoğraf ekle'}).focus();await page.keyboard.press('Enter');await expect(rows.nth(1).locator('img')).toHaveAttribute('src',/bcbcbcbc/);
  await rows.nth(1).getByRole('button',{name:'Kaldır',exact:true}).click();await page.getByRole('button',{name:'Adımı taslaktan kaldır'}).click();await expect(rows).toHaveCount(1);await expect(page.getByRole('button',{name:'Adım ekle',exact:true})).toBeFocused();
  await page.evaluate(()=>document.documentElement.style.fontSize='20px');expect(await page.locator('html').evaluate(e=>e.scrollWidth<=e.clientWidth+1)).toBe(true);
  await page.getByRole('button',{name:'Tarifi kaydet',exact:true}).click();await expect(page.locator('.detail-list .step-photo-preview img')).toHaveCount(1);
 }
});

test('500 real React step thumbnails remain lazy; large previews mount only on request',async({page})=>{
 const requests:string[]=[];page.on('request',r=>{if(r.url().includes('cover-fixture'))requests.push(r.url());});
 await page.setViewportSize({width:1024,height:768});await page.goto('/tests/ui/workspace.html?steps=1&stepPhotos=500&theme=dark');
 const cdp=await page.context().newCDPSession(page);await cdp.send('Performance.enable');
 const before=await cdp.send('Performance.getMetrics');const start=Date.now();await page.locator('.recipe-card-open').click();
 const images=page.locator('.detail-list .step-photo-preview img');await expect(images).toHaveCount(500);await expect(images.first()).toHaveJSProperty('naturalWidth',384);
 expect(await images.evaluateAll(nodes=>nodes.every(n=>n.getAttribute('loading')==='lazy'&&n.getAttribute('src')?.includes('variant=thumb')))).toBe(true);
 expect(requests.filter(url=>url.includes('variant=preview'))).toHaveLength(0);expect(new Set(requests).size).toBeLessThan(50);
 const after=await cdp.send('Performance.getMetrics');console.log('M5C2 browser 500-step detail metrics',JSON.stringify({elapsedMs:Date.now()-start,initialImageRequests:new Set(requests).size,before:before.metrics.filter(m=>['JSHeapUsedSize','Nodes'].includes(m.name)),after:after.metrics.filter(m=>['JSHeapUsedSize','Nodes'].includes(m.name))}));
 await page.getByRole('button',{name:'Düzenle',exact:true}).click();await expect(page.locator('[data-step-id]')).toHaveCount(500);await expect(page.locator('.step-photo-editor')).toHaveCount(0);await expect(page.locator('dialog')).toHaveCount(0);
 await page.locator('.step-advanced summary').first().click();await expect(page.locator('.step-photo-editor')).toHaveCount(1);await page.getByRole('button',{name:'1. adım fotoğrafını önizle',exact:true}).click();await expect(page.getByRole('dialog').locator('img')).toHaveAttribute('src',/variant=preview/);await expect.poll(()=>requests.filter(url=>url.includes('variant=preview')).length).toBe(1);await page.keyboard.press('Escape');await expect(page.getByRole('dialog')).not.toBeVisible();
});
