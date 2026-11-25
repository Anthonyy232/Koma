// node readme/capture.cjs <koma checkout> [<left> <top>] : Koma over the demo desktop, into readme/inputs/
// Build first (`cargo build -p koma-windows -p peek` in the checkout). Takes over the primary display for a few
// seconds a scheme: the demo page full screen, then Koma driven by tools/peek. <left> <top>: the primary
// display's top left in Koma's window, in pixels (0 0 unless a display sits left of or above it).
const { chromium } = require('playwright');
const { execFileSync } = require('child_process');
const path = require('path');
const { komaCheckout } = require('../lib.cjs');

const koma = komaCheckout(process.argv[2]);
const [left, top] = [process.argv[3], process.argv[4]].map(n => Number(n ?? 0));
const bin = name => path.join(koma, 'target/debug', name);
const out = path.join(__dirname, 'inputs');

async function capture(scheme) {
  const browser = await chromium.launch({ headless: false, args: ['--window-position=0,0'] });
  try {
    const page = await browser.newPage({ viewport: null });
    await page.goto(`file://${path.join(__dirname, 'demo.html')}${scheme === 'dark' ? '?dark' : ''}`);
    const cdp = await page.context().newCDPSession(page);
    const { windowId } = await cdp.send('Browser.getWindowForTarget');
    await cdp.send('Browser.setWindowBounds', { windowId, bounds: { windowState: 'fullscreen' } });
    // Until Chromium's full-screen notice has gone.
    await page.waitForTimeout(6000);
    const { x, y, scale } = await page.evaluate(() => {
      const { x, y } = document.querySelector('.desk').getBoundingClientRect();
      return { x, y, scale: devicePixelRatio };
    });
    // Desk points to the desktop's pixels.
    const at = (...points) => points.map((p, i) => Math.round((i % 2 ? top / scale + y + p : left / scale + x + p) * scale)).join(',');
    // A shot is cut in screen pixels, which start at the primary display.
    const shot = name => `shot:${path.join(out, `${name}-${scheme}.png`)}@${Math.round(x * scale)},${Math.round(y * scale)},${1600 * scale},${1000 * scale}`;
    execFileSync(bin('peek.exe'), [
      `run:${bin('Koma.exe')}`, 'wait:2500',
      `drag:${at(110, 100, 1170, 760)}`, 'wait:400', shot('select'),
      'key:a', `drag:${at(806, 584, 742, 522)}`,
      'key:e', `drag:${at(150, 370, 582, 418)}`,
      'key:x', `drag:${at(392, 612, 556, 676)}`,
      'key:t', `click:${at(812, 572)}`, 'wait:300', 'text:Park here', 'key:enter',
      // Off the bar and the marks, so no tooltip or hover shows.
      `move:${at(1100, 700)}`, 'wait:400', shot('marks'),
      'key:escape', 'wait:300', 'kill',
    ], { stdio: 'inherit' });
  } finally {
    await browser.close();
  }
}

(async () => {
  for (const scheme of ['light', 'dark']) await capture(scheme);
})();
