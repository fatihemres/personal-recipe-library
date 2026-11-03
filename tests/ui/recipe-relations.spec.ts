import {test,expect} from '@playwright/test';
test('actual relations panel remains responsive, labels inactive links and confirms parent changes accessibly',async({page})=>{
 for(const theme of ['light','dark']){
  await page.goto(`/tests/ui/workspace.html?steps=1&theme=${theme}`);await page.locator('.recipe-card-open').click();const panel=page.getByRole('region',{name:'Varyasyonlar ve ilişkili tarifler'});await expect(panel.getByText(/Arşivde/)).toBeVisible();expect(await panel.getByRole('searchbox').evaluate(e=>parseFloat(getComputedStyle(e).borderTopWidth))).toBeGreaterThanOrEqual(1);expect(await panel.getByRole('searchbox').evaluate(e=>e.getBoundingClientRect().height)).toBeGreaterThanOrEqual(44);
  for(const width of [520,640,768,1024,1440,1920]){await page.setViewportSize({width,height:width===520?600:768});expect(await page.locator('html').evaluate(e=>e.scrollWidth<=e.clientWidth+1)).toBe(true);const fits=await panel.evaluate(e=>{const p=e.getBoundingClientRect();return Array.from(e.querySelectorAll('input,button')).every(c=>{const r=c.getBoundingClientRect();return r.width===0||r.left>=p.left-1&&r.right<=p.right+1;});});expect(fits).toBe(true);if([520,768,1440].includes(width))await panel.screenshot({path:test.info().outputPath(`relations-${theme}-${width}.png`)});}
  await panel.getByRole('searchbox').fill('tarif');const choose=panel.getByRole('button',{name:/Üst tarif olarak seç/});await choose.waitFor();await choose.focus();await page.keyboard.press('Enter');const dialog=page.getByRole('dialog');await expect(dialog.getByRole('button',{name:'Vazgeç'})).toBeFocused();await page.keyboard.press('Escape');await expect(dialog).not.toBeVisible();await expect(choose).toBeFocused();await page.setViewportSize({width:640,height:600});await page.evaluate(()=>{document.documentElement.style.fontSize='20px';});expect(await page.locator('html').evaluate(e=>e.scrollWidth<=e.clientWidth+1)).toBe(true);
 }
});

test('actual two-stage purge dialog requires all dependencies and handles stale review without unsafe retry',async({page})=>{
 for(const theme of ['light','dark']){
  await page.goto(`/tests/ui/workspace.html?purge=1&theme=${theme}`);const dialog=page.getByRole('dialog');await expect(dialog.getByRole('button',{name:'Bağları inceledim, devam et'})).toBeDisabled();await dialog.getByRole('button',{name:'Daha fazla bağ'}).click();await expect(dialog.getByRole('button',{name:'Bağları inceledim, devam et'})).toBeEnabled();await dialog.getByRole('button',{name:'Bağları inceledim, devam et'}).click();await expect(dialog.getByRole('button',{name:'Vazgeç'})).toBeFocused();
  for(const width of [520,768,1440]){await page.setViewportSize({width,height:600});expect(await page.locator('html').evaluate(e=>e.scrollWidth<=e.clientWidth+1)).toBe(true);await expect(dialog.getByRole('button',{name:'Bağları ayır ve kalıcı sil'})).toBeInViewport();await dialog.screenshot({path:test.info().outputPath(`purge-${theme}-${width}.png`)});}
  await dialog.getByRole('button',{name:'Bağları ayır ve kalıcı sil'}).click();await expect(dialog.getByRole('alert')).toContainText('Silme yapılmadı');await expect(dialog.getByRole('button',{name:'Kalıcı olarak sil'})).toBeDisabled();await expect(dialog.getByRole('button',{name:'Güncel bağımlılıkları yeniden incele'})).toBeVisible();
 }
});

// Different keyed detail sections must not share recipe.id as a sibling key.
test('linked recipe navigation replaces detail sections without retaining prior recipe panels',async({page})=>{
 await page.goto('/tests/ui/workspace.html?steps=1');await page.locator('.recipe-card-open').click();
 for(let cycle=0;cycle<3;cycle++){
  const panel=page.getByRole('region',{name:'Varyasyonlar ve ilişkili tarifler'});
  await expect(panel).toHaveCount(1);await panel.getByRole('button',{name:/Tarifi aç/}).click();
  await expect(page.getByRole('heading',{name:'Hedef tarif',exact:true})).toBeVisible();
  await expect(panel).toHaveCount(1);await expect(page.getByRole('region',{name:'Sürüm geçmişi',exact:true})).toHaveCount(1);
  await expect(page.getByRole('region',{name:'Özel Markdown notu',exact:true})).toHaveCount(1);
  await page.getByRole('button',{name:'Kütüphaneye dön'}).click();await page.locator('.recipe-card-open').click();
 }
});
