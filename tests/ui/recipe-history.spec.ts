import { test,expect } from '@playwright/test';
test('real history component supports keyboard, exact comparison and responsive restore confirmation',async({page})=>{
 for(const theme of ['light','dark']) {
  await page.goto(`/tests/ui/workspace.html?steps=1&theme=${theme}`);
  await page.locator('.recipe-card-open').click();
  const history=page.getByRole('region',{name:'Sürüm geçmişi'});
  const open=history.getByRole('button',{name:'Geçmişi aç'});await open.focus();await page.keyboard.press('Enter');
  await history.getByRole('button',{name:'V1 · Sürümü görüntüle'}).click();
  await history.getByRole('button',{name:'Karşılaştır',exact:true}).click();
  await expect(history.getByText('"1.250"',{exact:true})).toBeVisible();
  await expect(history.getByText('"1.25"',{exact:true})).toBeVisible();
  for(const width of [520,640,768,1024,1440,1920]) {
   await page.setViewportSize({width,height:width===520?600:768});
   expect(await page.locator('html').evaluate(e=>e.scrollWidth<=e.clientWidth+1)).toBe(true);
   const geometry=await history.evaluate(e=>{
    const p=e.getBoundingClientRect();return Array.from(e.querySelectorAll('button,select')).filter(c=>c.getBoundingClientRect().width>0).map(c=>{const b=c.getBoundingClientRect();return b.left>=p.left-1&&b.right<=p.right+1;});
   });expect(geometry.every(Boolean)).toBe(true);
   if([520,768,1440].includes(width))await history.screenshot({path:test.info().outputPath(`history-${theme}-${width}.png`)});
  }
  await page.setViewportSize({width:640,height:600});await page.evaluate(()=>{document.documentElement.style.fontSize='20px';});
  expect(await page.locator('html').evaluate(e=>e.scrollWidth<=e.clientWidth+1)).toBe(true);
  await history.getByRole('button',{name:'Bu sürümü geri yükle'}).click();
  const dialog=page.getByRole('dialog');await expect(dialog.getByRole('button',{name:'Vazgeç'})).toBeFocused();
  await page.keyboard.press('Escape');await expect(dialog).not.toBeVisible();
  await expect(history.getByRole('button',{name:'Bu sürümü geri yükle'})).toBeFocused();
 }
});
