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
