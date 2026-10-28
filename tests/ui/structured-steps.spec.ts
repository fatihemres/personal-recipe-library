import { expect,test } from '@playwright/test';

test('actual advanced recipe editor fits desktop containers and preserves accessible step interactions',async({page})=>{
 for(const theme of ['light','dark']){
  await page.goto(`/tests/ui/workspace.html?steps=1&theme=${theme}`);
  await page.locator('.recipe-card-open').click();
  await page.getByRole('button',{name:'Düzenle',exact:true}).click();
  const row=page.locator('.step-row').first();
  await expect(row.locator('details')).not.toHaveAttribute('open');
  const summary=row.locator('summary');await summary.focus();await page.keyboard.press('Enter');
  await expect(row.locator('details')).toHaveAttribute('open','');
  await page.keyboard.press('Tab');await expect(row.getByLabel('Adım başlığı')).toBeFocused();
  await row.getByLabel('Adım başlığı').fill('Ön hazırlık ve uzun Türkçe adım başlığı');
  await row.getByLabel('Süre (dakika, tam sayı)',{exact:true}).fill('0');
  await row.getByLabel('Sıcaklık (tam sayı)',{exact:true}).fill('180');
  await row.getByLabel('Sıcaklık birimi').selectOption('C');
  await row.getByRole('checkbox').check();
  for(const width of [520,640,768,1024,1440,1920]){
   await page.setViewportSize({width,height:768});
   const geometry=await row.evaluate(element=>{
    const bounds=element.getBoundingClientRect();
    const controls=Array.from(element.querySelectorAll<HTMLElement>('input,textarea,select,button')).filter(e=>e.getBoundingClientRect().width>0).map(e=>{const r=e.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom};});
    return {width:document.documentElement.clientWidth,scroll:document.documentElement.scrollWidth,left:bounds.left,right:bounds.right,controls};
   });
   expect(geometry.scroll,`${theme} ${width}`).toBeLessThanOrEqual(geometry.width+1);
   for(const box of geometry.controls){expect(box.left).toBeGreaterThanOrEqual(geometry.left);expect(box.right).toBeLessThanOrEqual(geometry.right+1);}
   for(let i=0;i<geometry.controls.length;i++)for(let j=i+1;j<geometry.controls.length;j++){
    const a=geometry.controls[i],b=geometry.controls[j];expect(a.right<=b.left+1||b.right<=a.left+1||a.bottom<=b.top+1||b.bottom<=a.top+1).toBe(true);
   }
   const heights=await row.locator('.form-grid').first().locator('input,select').evaluateAll(controls=>controls.map(control=>control.getBoundingClientRect().height));
   expect(Math.max(...heights)-Math.min(...heights)).toBeLessThanOrEqual(2);
   if([520,768,1440].includes(width)) {
    await row.screenshot({path:test.info().outputPath(`advanced-${theme}-${width}.png`)});
    await summary.scrollIntoViewIfNeeded();
    await page.screenshot({path:test.info().outputPath(`editor-viewport-${theme}-${width}.png`)});
   }
  }
  for(const height of [560,600,900]){
   await page.setViewportSize({width:520,height});await expect(row.getByRole('checkbox')).toBeChecked();
   await row.getByRole('button',{name:'1. Adım: Aşağı taşı'}).scrollIntoViewIfNeeded();
   await expect(row.getByRole('button',{name:'1. Adım: Aşağı taşı'})).toBeInViewport();
  }
  await page.evaluate(()=>{document.documentElement.style.fontSize='20px';});
  await page.setViewportSize({width:640,height:600});
  await row.screenshot({path:test.info().outputPath(`advanced-${theme}-large-text.png`)});
  expect(await page.locator('html').evaluate(e=>e.scrollWidth<=e.clientWidth+1)).toBe(true);
  await page.getByRole('button',{name:'1. Adım: Aşağı taşı'}).click();
  const moved=page.locator('[data-step-id="66666666-6666-4666-8666-666666666666"]');
  await expect(moved.getByRole('checkbox')).toBeChecked();await expect(moved.getByLabel('Adım başlığı')).toHaveValue('Ön hazırlık ve uzun Türkçe adım başlığı');
  await page.locator('.ingredient-row').getByRole('button',{name:'Kaldır'}).click();
  await expect(page.getByRole('status').filter({hasText:'Kaldırılan malzeme satırının'})).toBeVisible();
  await expect(moved.getByRole('checkbox')).toHaveCount(0);
 }
});
