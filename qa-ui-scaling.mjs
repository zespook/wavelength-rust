import assert from 'node:assert/strict';

const viewportWidths = [320, 360, 375, 390, 430, 560, 620, 735, 736, 768, 900, 1024, 1128, 1312, 1920, 3840];
const routeChecks = [
  { path: '/player', selector: '.home-greeting h1', widths: [320, 736, 1312] },
  { path: '/search', selector: '.search-page-form', widths: [320, 736, 1312] },
  { path: '/library', selector: '.library-page-filters', widths: [320, 736, 1312] },
  { path: '/now-playing', selector: '.now-playing-art', widths: [320, 736, 1312] },
  { path: '/settings', selector: '.settings-page-heading', widths: [320, 736, 1312] },
  { path: '/queue', selector: '.mobile-queue-page', widths: [320] },
];

function expectedScale(width) {
  if (width <= 735) return 0.88;
  if (width <= 1128) return 0.9;
  return 0.92;
}

async function setViewport(page, width, height = 900) {
  await page.setViewportSize({ width, height });
  const scale = expectedScale(width);
  await page.waitForFunction((expected) => {
    const root = document.querySelector('#app');
    return root && Math.abs(Number(getComputedStyle(root).zoom) - expected) < 0.001;
  }, scale);
}

async function clickAtScaledPoint(page, locator) {
  const point = await locator.evaluate((element) => {
    const zoom = Number(getComputedStyle(document.querySelector('#app')).zoom);
    const bounds = element.getBoundingClientRect();
    return {
      x: (bounds.left + bounds.width / 2) * zoom,
      y: (bounds.top + bounds.height / 2) * zoom,
    };
  });
  const viewport = await page.evaluate(() => ({ width: innerWidth, height: innerHeight }));
  assert.ok(point.x >= 0 && point.x <= viewport.width && point.y >= 0 && point.y <= viewport.height, 'scaled click target is outside the viewport');
  await page.mouse.click(point.x, point.y);
}

async function measure(page) {
  return page.evaluate(() => {
    const zoom = Number(getComputedStyle(document.querySelector('#app')).zoom);
    const rect = (element) => {
      const bounds = element.getBoundingClientRect();
      return {
        left: bounds.left * zoom,
        right: bounds.right * zoom,
        top: bounds.top * zoom,
        bottom: bounds.bottom * zoom,
        width: bounds.width * zoom,
      };
    };
    const visible = (element) => {
      const style = getComputedStyle(element);
      const bounds = element.getBoundingClientRect();
      return style.display !== 'none' && style.visibility !== 'hidden' && bounds.width > 0 && bounds.height > 0;
    };
    const bar = document.querySelector('.player-bar');
    const sections = ['.vinyl-player', '.player-center', '.player-tools'].map((selector) => document.querySelector(selector));
    const sectionBounds = sections.map(rect);
    const tools = sections[2];
    const playerControlsFit = [...tools.children].filter(visible).every((element) => {
      const control = rect(element);
      return control.left >= sectionBounds[2].left - 1 && control.right <= sectionBounds[2].right + 1;
    });
    const content = document.querySelector('.content-inner');
    const contentBounds = rect(content);
    const isInsideContent = (element) => {
      if (!element) return false;
      const bounds = rect(element);
      return bounds.left >= contentBounds.left - 1 && bounds.right <= contentBounds.right + 1;
    };
    const byLabel = (label) => tools.querySelector(`[aria-label="${label}"]`);
    const volume = tools.querySelector('.volume-control');
    const mobileNav = document.querySelector('.mobile-bottom-nav');
    const barBounds = rect(bar);
    const navBounds = rect(mobileNav);

    return {
      width: innerWidth,
      zoom,
      documentOverflow: Math.max(document.documentElement.scrollWidth, document.body.scrollWidth) > document.documentElement.clientWidth + 1,
      bar: barBounds,
      sections: sectionBounds,
      gaps: [sectionBounds[1].left - sectionBounds[0].right, sectionBounds[2].left - sectionBounds[1].right],
      playerControlsFit,
      controls: {
        party: visible(byLabel('Open party details')),
        queue: visible(byLabel('Show queue')),
        details: visible(byLabel('Show song details')),
        fullscreen: visible(byLabel('Fullscreen')),
        volume: visible(volume),
      },
      mobileNavVisible: visible(mobileNav),
      mobileNavButtons: [...mobileNav.querySelectorAll('button')].filter(visible).length,
      mobileNavBottom: navBounds.bottom,
      searchFormInsideContent: isInsideContent(document.querySelector('.search-page-form')),
      trackTableInsideContent: isInsideContent(document.querySelector('.track-table')),
    };
  });
}

function assertResponsiveLayout(metrics) {
  const { width } = metrics;
  assert.equal(metrics.documentOverflow, false, `${width}px: document overflows horizontally`);
  assert.ok(Math.abs(metrics.bar.left) <= 1, `${width}px: player bar is offset from the left edge`);
  assert.ok(Math.abs(metrics.bar.right - width) <= 1, `${width}px: player bar does not span the viewport`);
  assert.equal(metrics.playerControlsFit, true, `${width}px: player controls overflow their section`);
  assert.equal(metrics.searchFormInsideContent, true, `${width}px: search form escapes the content area`);
  assert.equal(metrics.trackTableInsideContent, true, `${width}px: track table escapes the content area`);

  if (width >= 736) {
    const expectedGap = Math.min(64, Math.max(16, width * 0.04)) * metrics.zoom;
    assert.ok(Math.abs(metrics.sections[0].width / metrics.bar.width - 0.27) < 0.005, `${width}px: left section is not 27%`);
    assert.ok(Math.abs(metrics.sections[2].width / metrics.bar.width - 0.27) < 0.005, `${width}px: right section is not 27%`);
    for (const gap of metrics.gaps) {
      assert.ok(Math.abs(gap - expectedGap) <= 1, `${width}px: player gap is not fluidly scaled`);
    }
    for (const control of Object.keys(metrics.controls)) {
      assert.equal(metrics.controls[control], true, `${width}px: ${control} control is hidden before mobile`);
    }
    assert.equal(metrics.mobileNavVisible, false, `${width}px: mobile navigation is unexpectedly visible`);
  } else {
    assert.equal(metrics.mobileNavVisible, true, `${width}px: mobile navigation is missing`);
    assert.ok(metrics.mobileNavButtons >= 4, `${width}px: mobile navigation controls are missing`);
    assert.ok(Math.abs(metrics.mobileNavBottom - 900) <= 1, `${width}px: mobile navigation is not anchored to the viewport bottom`);
    for (const control of ['queue', 'details', 'fullscreen']) {
      assert.equal(metrics.controls[control], true, `${width}px: essential mobile ${control} control is hidden`);
    }
    assert.equal(metrics.controls.volume, false, `${width}px: volume control should collapse at the mobile breakpoint`);
  }
}

export default async function run(page) {
  const origin = new URL(page.url()).origin;
  await page.emulateMedia({ reducedMotion: 'no-preference' });
  await page.goto(`${origin}/search`);
  await page.waitForSelector('.track-row');

  const responsiveResults = [];
  for (const width of viewportWidths) {
    await setViewport(page, width);
    const metrics = await measure(page);
    assert.ok(Math.abs(metrics.zoom - expectedScale(width)) < 0.001, `${width}px: incorrect responsive app scale`);
    assertResponsiveLayout(metrics);
    responsiveResults.push({ width, zoom: metrics.zoom, gaps: metrics.gaps.map(Math.round) });
  }

  await setViewport(page, 1312);
  await page.setViewportSize({ width: 1024, height: 900 });
  const transitionSamples = await page.evaluate(async () => {
    const values = [];
    for (let frame = 0; frame < 8; frame += 1) {
      await new Promise(requestAnimationFrame);
      values.push(Number(getComputedStyle(document.querySelector('#app')).zoom));
    }
    return values;
  });
  assert.ok(transitionSamples.some((scale) => scale > 0.901 && scale < 0.919), 'responsive scale does not transition between desktop and tablet');
  await setViewport(page, 1024);

  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.setViewportSize({ width: 1312, height: 900 });
  await page.waitForFunction(() => Number(getComputedStyle(document.querySelector('#app')).zoom) === 0.92);
  await page.setViewportSize({ width: 1024, height: 900 });
  const reducedMotion = await page.evaluate(() => ({
    zoom: Number(getComputedStyle(document.querySelector('#app')).zoom),
    transitionDuration: getComputedStyle(document.querySelector('#app')).transitionDuration,
  }));
  assert.equal(reducedMotion.zoom, 0.9, 'reduced-motion mode has the wrong tablet scale');
  assert.equal(reducedMotion.transitionDuration, '0s', 'reduced-motion mode still animates app scaling');
  await page.emulateMedia({ reducedMotion: 'no-preference' });

  for (const route of routeChecks) {
    for (const width of route.widths) {
      await setViewport(page, width);
      await page.goto(`${origin}${route.path}`);
      await page.waitForSelector(route.selector);
      const marker = await page.locator(route.selector).boundingBox();
      assert.ok(marker && marker.width > 0 && marker.height > 0, `${route.path} did not render at ${width}px`);
      const metrics = await measure(page);
      assert.equal(metrics.documentOverflow, false, `${route.path} overflows horizontally at ${width}px`);
      assert.ok(Math.abs(metrics.bar.left) <= 1 && Math.abs(metrics.bar.right - width) <= 1, `${route.path} player bar is not viewport-aligned at ${width}px`);
    }
  }

  await setViewport(page, 390);
  await page.goto(`${origin}/search`);
  await page.waitForSelector('.search-page-form');
  const mobileNav = page.locator('.mobile-bottom-nav');
  await clickAtScaledPoint(page, mobileNav.getByRole('button', { name: 'Library' }));
  await page.waitForSelector('.library-page-filters');
  await clickAtScaledPoint(page, mobileNav.getByRole('button', { name: 'Search' }));
  await page.waitForSelector('.search-page-form');
  await clickAtScaledPoint(page, page.locator('.vinyl-player'));
  await page.waitForSelector('.now-playing-art');
  assert.ok(page.url().endsWith('/now-playing'), 'player bar does not open the now-playing view');

  await setViewport(page, 1312, 959);
  return {
    checkedWidths: responsiveResults.length,
    responsiveResults,
    checkedRoutes: routeChecks.length,
    checkedTransition: true,
    checkedReducedMotion: true,
    checkedMobileTabsAndPlayer: true,
  };
}