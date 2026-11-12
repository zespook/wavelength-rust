export default async function run(page) {
  await page.goto('http://127.0.0.1:5173/search');
  await page.waitForSelector('.track-row');
  await page.locator('.track-row').nth(0).click();
  await page.waitForTimeout(150);
  return {
    selectedRows: await page.locator('.track-row.selected').count(),
    detailsRails: await page.locator('.details-rail').count(),
    bodyOverflow: await page.evaluate(() => document.documentElement.scrollWidth > document.documentElement.clientWidth),
  };
}
