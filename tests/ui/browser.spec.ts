import { expect, test } from '@playwright/test';
// No native IPC mocking: a plain browser must accurately explain the unavailable service.
test('browser shell explains desktop requirement and keyboard navigation works', async ({ page }) => {
  await page.goto('/');
  await expect(page).toHaveTitle('Tarif Atlası');
  await expect(page.locator('html')).toHaveAttribute('lang', 'tr');
  await expect(page.getByRole('alert')).toContainText('masaüstü uygulamasını açın');
  await page.keyboard.press('Tab');
  await expect(page.getByRole('link', { name: 'İçeriğe geç' })).toBeFocused();
  await page.getByRole('button', { name: 'Ayarlar', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Ayarlar', exact: true })).toHaveAttribute('aria-current', 'page');
  await expect(page.getByRole('button', { name: 'Yeniden dene' })).toBeVisible();
});
test('narrow screen reveals sidebar through accessible disclosure', async ({ page }) => {
  await page.setViewportSize({ width: 560, height: 760 }); await page.goto('/');
  const toggle = page.getByRole('button', { name: 'Menüyü aç / kapat' });
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await toggle.click(); await expect(toggle).toHaveAttribute('aria-expanded', 'true');
  await expect(page.getByRole('navigation')).toBeVisible();
  await expect(page.locator('body')).toHaveJSProperty('scrollWidth', 560);
});
test('collection navigation stays discoverable during a storage error', async ({ page }) => {
  await page.goto('/');
  const collections=page.getByRole('button',{name:'Koleksiyonlar',exact:true});
  await expect(collections).toBeVisible();
  await collections.click();
  await expect(collections).toHaveAttribute('aria-current','page');
  await expect(page.getByRole('alert')).toContainText('masaüstü uygulamasını açın');
});
test('dashboard navigation is available while storage recovery remains honest',async({page})=>{
  await page.goto('/');
  const dashboard=page.getByRole('button',{name:'Özet ve istatistikler',exact:true});
  await expect(dashboard).toBeVisible();await dashboard.click();
  await expect(dashboard).toHaveAttribute('aria-current','page');
  await expect(page.getByRole('alert')).toContainText('masaüstü uygulamasını açın');
});
test('collection and note layouts fit narrow, medium and wide content areas',async({page})=>{
  await page.goto('/');
  await expect(page.getByRole('alert')).toContainText('masaüstü uygulamasını açın');
  await page.evaluate(()=>{
    document.querySelector('#root')!.setAttribute('hidden','');
    const fixture=document.createElement('div');
    fixture.className='app-shell';
    fixture.innerHTML=`<aside class="sidebar"></aside><div class="workspace"><main class="main-content">
      <section class="panel recipe-collections"><h2>Koleksiyonlar</h2><form class="recipe-collection-form">
        <label>Koleksiyon adı<input aria-label="Koleksiyon adı"></label>
        <div class="recipe-collection-actions"><button type="submit">Koleksiyon oluştur</button><button type="button">Vazgeç</button></div>
      </form></section>
      <section class="panel private-note"><h2>Özel Markdown notu</h2>
        <div class="recipe-actions"><button>Yaz</button><button>Önizleme</button></div>
        <label class="note-editor">Özel Markdown notu<textarea aria-label="Özel Markdown notu"></textarea></label>
        <div class="recipe-actions"><button>Notu kaydet</button><button>Notu temizle</button></div>
      </section></main></div>`;
    document.body.append(fixture);
    fixture.querySelector('textarea')!.value=Array.from({length:80},(_,i)=>`Satır ${i+1}`).join('\n');
  });
  for(const width of [320,560,920,1440]){
    await page.setViewportSize({width,height:760});
    for(const dark of [false,true]){
      await page.locator('html').evaluate((element,darkMode)=>element.classList.toggle('dark',darkMode),dark);
      await expect(page.getByRole('button',{name:'Koleksiyon oluştur'})).toBeVisible();
      await expect(page.getByRole('button',{name:'Notu kaydet'})).toBeVisible();
      await expect(page.getByRole('button',{name:'Notu temizle'})).toBeVisible();
      await expect(page.getByRole('button',{name:'Yaz'})).toBeVisible();
      await expect(page.getByRole('button',{name:'Önizleme'})).toBeVisible();
      const dimensions=await page.evaluate(()=>{
        const panel=document.querySelector('.recipe-collections')!.getBoundingClientRect();
        const input=document.querySelector('.recipe-collection-form input')!.getBoundingClientRect();
        const action=document.querySelector('.recipe-collection-actions')!.getBoundingClientRect();
        const editor=document.querySelector('.note-editor textarea') as HTMLTextAreaElement;
        const note=document.querySelector('.private-note')!.getBoundingClientRect();
        return {viewport:innerWidth,documentWidth:document.documentElement.scrollWidth,
          panelLeft:panel.left,panelRight:panel.right,inputLeft:input.left,inputRight:input.right,
          actionLeft:action.left,actionRight:action.right,inputBottom:input.bottom,actionTop:action.top,
          editorHeight:editor.clientHeight,editorScrollHeight:editor.scrollHeight,
          editorRight:editor.getBoundingClientRect().right,noteRight:note.right,
          editorOverflow:getComputedStyle(editor).overflowY};
      });
      expect(dimensions.documentWidth).toBeLessThanOrEqual(dimensions.viewport+1);
      expect(dimensions.inputLeft).toBeGreaterThanOrEqual(dimensions.panelLeft);
      expect(dimensions.inputRight).toBeLessThanOrEqual(dimensions.panelRight);
      expect(dimensions.actionRight).toBeLessThanOrEqual(dimensions.panelRight);
      expect(dimensions.actionLeft>=dimensions.inputRight || dimensions.actionTop>=dimensions.inputBottom).toBe(true);
      expect(dimensions.editorRight).toBeLessThanOrEqual(dimensions.noteRight);
      expect(dimensions.editorHeight).toBeLessThanOrEqual(220);
      expect(dimensions.editorScrollHeight).toBeGreaterThan(dimensions.editorHeight);
      expect(dimensions.editorOverflow).toBe('auto');
    }
  }
  await page.getByRole('textbox',{name:'Koleksiyon adı'}).focus();
  await page.keyboard.press('Tab');
  await expect(page.getByRole('button',{name:'Koleksiyon oluştur'})).toBeFocused();
});
