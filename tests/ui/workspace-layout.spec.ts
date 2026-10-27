import { expect, test, type Page } from '@playwright/test';

const widths=[360,480,520,640,768,1024,1440,1920];
const themes=['light','dark'] as const;

async function navigate(page:Page,name:string){
  const menu=page.getByRole('button',{name:'Menüyü aç / kapat'});
  if(await menu.isVisible()) await menu.click();
  await page.getByRole('navigation',{name:'Ana gezinme'}).getByRole('button',{name,exact:true}).click();
}

async function expectContained(page:Page, selector:string, width:number, theme:string) {
  const bounds=await page.locator(selector).evaluate(element=>({
    client:element.clientWidth,scroll:element.scrollWidth,
    left:element.getBoundingClientRect().left,right:element.getBoundingClientRect().right,
  }));
  expect(bounds.scroll,`${selector} overflows at ${width}px in ${theme}`).toBeLessThanOrEqual(bounds.client+1);
  expect(bounds.left,`${selector} escapes left at ${width}px in ${theme}`).toBeGreaterThanOrEqual(-1);
  expect(bounds.right,`${selector} escapes right at ${width}px in ${theme}`).toBeLessThanOrEqual(width+1);
}

async function expectStructuredForm(page:Page,panel:string,width:number,theme:string) {
  const geometry=await page.locator(panel).evaluate(element=>{
    const field=element.querySelector('.workspace-field')!;
    const label=field.querySelector('span')!.getBoundingClientRect();
    const input=field.querySelector('input')!.getBoundingClientRect();
    const actions=element.querySelector('.workspace-form-actions')!.getBoundingClientRect();
    const primary=element.querySelector<HTMLButtonElement>('.workspace-form-actions button')!;
    const button=primary.getBoundingClientRect();
    const parent=element.getBoundingClientRect();
    return {labelBottom:label.bottom,inputTop:input.top,inputLeft:input.left,inputRight:input.right,
      actionLeft:actions.left,actionTop:actions.top,actionRight:actions.right,buttonRight:button.right,
      buttonScroll:primary.scrollWidth,buttonClient:primary.clientWidth,parentLeft:parent.left,parentRight:parent.right};
  });
  expect(geometry.inputTop,`${panel} label above input at ${width}px ${theme}`).toBeGreaterThanOrEqual(geometry.labelBottom+3);
  expect(geometry.inputLeft).toBeGreaterThanOrEqual(geometry.parentLeft);
  expect(geometry.inputRight).toBeLessThanOrEqual(geometry.parentRight);
  expect(geometry.actionRight).toBeLessThanOrEqual(geometry.parentRight);
  expect(geometry.inputRight<=geometry.actionLeft || geometry.inputTop<geometry.actionTop).toBe(true);
  expect(geometry.buttonScroll).toBeLessThanOrEqual(geometry.buttonClient+1);
}

async function expectSeparateControls(page:Page,panel:string){
  const boxes=await page.locator(`${panel} .workspace-list-row`).first().locator('button').evaluateAll(buttons=>buttons.map(button=>{
    const box=button.getBoundingClientRect();return {left:box.left,right:box.right,top:box.top,bottom:box.bottom};
  }));
  for(let first=0;first<boxes.length;first++) for(let second=first+1;second<boxes.length;second++){
    const a=boxes[first],b=boxes[second];
    expect(a.right<=b.left+1 || b.right<=a.left+1 || a.bottom<=b.top+1 || b.bottom<=a.top+1).toBe(true);
  }
}

test('real recipe workspace components stay contained across themes and desktop widths',async({page})=>{
  for(const theme of themes){
    await page.goto(`/tests/ui/workspace.html?theme=${theme}`);
    await expect(page.locator('.saved-filter-panel .workspace-list-row')).toBeVisible();
    for(const width of widths){
      await page.setViewportSize({width,height:768});
      await expectContained(page,'html',width,theme);
      await expectContained(page,'.main-content',width,theme);
      await expectContained(page,'.saved-filter-panel',width,theme);
      await expectContained(page,'.recipe-search-filters',width,theme);
      await expectContained(page,'.recipe-grid',width,theme);
      await expectStructuredForm(page,'.saved-filter-panel',width,theme);
      await expectContained(page,'.saved-filter-panel .workspace-list-row',width,theme);
      await expectSeparateControls(page,'.saved-filter-panel');
      if([360,480,520,768,1024,1440,1920].includes(width)){
        await page.locator('.saved-filter-panel').screenshot({path:test.info().outputPath(`saved-filters-${theme}-${width}.png`)});
      }
    }
    for(const height of [560,600,900]){
      await page.setViewportSize({width:640,height});
      await expectContained(page,'html',640,`${theme} ${height}px tall`);
      await expectStructuredForm(page,'.saved-filter-panel',640,`${theme} ${height}px tall`);
    }
    await page.setViewportSize({width:480,height:600});
    await navigate(page,'Koleksiyonlar');
    await expect(page.locator('.recipe-collections .workspace-list-row')).toBeVisible();
    await expectContained(page,'.recipe-collections',480,theme);
    await expectStructuredForm(page,'.recipe-collections',480,theme);
    await expectSeparateControls(page,'.recipe-collections');
    await page.locator('.recipe-collections').screenshot({path:test.info().outputPath(`collections-${theme}-480.png`)});
    await navigate(page,'Tarif kütüphanesi');
    await page.getByRole('button',{name:'Liste'}).click();
    await expectContained(page,'.recipe-list-row',480,theme);
    await page.locator('.recipe-list-row').screenshot({path:test.info().outputPath(`recipe-list-${theme}-480.png`)});
    await page.locator('.recipe-card-open').first().click();
    await expect(page.locator('.private-note textarea')).toBeVisible();
    await expectContained(page,'.private-note',480,theme);
    const note=await page.locator('.private-note textarea').evaluate(element=>({client:element.clientHeight,scroll:element.scrollHeight}));
    expect(note.scroll).toBeGreaterThan(note.client);
    expect(note.client).toBeLessThanOrEqual(220);
    await page.locator('.private-note').screenshot({path:test.info().outputPath(`private-note-${theme}-480.png`)});
    await navigate(page,'Özet ve istatistikler');
    await expect(page.locator('.dashboard-metrics')).toBeVisible();
    await expectContained(page,'.dashboard-metrics',480,theme);
    await expectContained(page,'.dashboard-sections',480,theme);
    await page.locator('.dashboard-sections').screenshot({path:test.info().outputPath(`dashboard-${theme}-480.png`)});
  }
});

test('actual saved filter, collection and dialog controls keep focus order at text zoom',async({page})=>{
  await page.setViewportSize({width:640,height:600});
  await page.goto('/tests/ui/workspace.html?theme=dark');
  await expect(page.locator('.saved-filter-panel .workspace-list-row')).toBeVisible();
  await page.evaluate(()=>{document.documentElement.style.fontSize='20px';});
  await expectContained(page,'html',640,'dark, enlarged text');
  await expectStructuredForm(page,'.saved-filter-panel',640,'dark, enlarged text');
  const input=page.getByRole('textbox',{name:'Filtre adı'});
  await input.fill('Deneme filtresi');await input.focus();await page.keyboard.press('Tab');
  await expect(page.getByRole('button',{name:'Filtreyi kaydet'})).toBeFocused();
  const focusStyle=await page.getByRole('button',{name:'Filtreyi kaydet'}).evaluate(button=>getComputedStyle(button).outlineStyle);
  expect(focusStyle).not.toBe('none');
  await navigate(page,'Koleksiyonlar');
  await expectStructuredForm(page,'.recipe-collections',640,'dark, enlarged text');
  await page.getByRole('button',{name:/Koleksiyonu sil/}).click();
  await expect(page.getByRole('dialog')).toBeVisible();
  await expectContained(page,'.confirm-dialog',640,'dark, enlarged text');
  await expect(page.getByRole('button',{name:'Vazgeç'})).toBeFocused();
  await page.keyboard.press('Escape');
  await expect(page.getByRole('dialog')).toHaveCount(0);
});
