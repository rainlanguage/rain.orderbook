const { coreSettings } = require('../common/settings');

describe('App Launch', () => {
  beforeEach(async () => {
    await browser.execute(() => {
      localStorage.clear();
    });
    await browser.refresh();
  })

  it('loads sidebar', async () => {
    await $("body > aside");
  });

  it('opens Settings page on first launch', async () => {
    const breadcrumbPageTitle = await $("span[data-testid=breadcrumb-page-title]").getText();
    expect(breadcrumbPageTitle).toEqual("Settings");
  });

  it('opens Orders page if settings have valid network & orderbook', async () => {
    await browser.execute((val) => {
      localStorage.setItem("settings", val);
    }, [coreSettings]);
    await browser.navigateTo('tauri://localhost/');
    await browser.refresh();

    const breadcrumbPageTitle = await $("span[data-testid=breadcrumb-page-title]").getText();
    expect(breadcrumbPageTitle).toEqual("Orders");
  });
});