// Shared by releases/banner.cjs and readme/build.cjs.
const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

const font = (file, style) => `@font-face { font-family: 'Instrument Serif'; font-style: ${style}; font-weight: 400;
  src: url(data:font/woff2;base64,${fs.readFileSync(path.join(__dirname, 'fonts', file)).toString('base64')}) format('woff2'); }`;
const fonts = `<style>${font('InstrumentSerif-Regular.woff2', 'normal')}\n${font('InstrumentSerif-Italic.woff2', 'italic')}</style>`;

// The master logos live on main; inline one without its fixed size and title.
const logo = (koma, file = 'logo.svg') => fs.readFileSync(path.join(koma, 'assets', file), 'utf8')
  .replace(/<title>.*?<\/title>/, '').replace('width="256" height="256"', '');

function komaCheckout(arg) {
  const dir = path.resolve(arg ?? '');
  if (!arg || !fs.existsSync(path.join(dir, 'assets/logo.svg'))) throw new Error('Pass the path of a Koma checkout of main');
  return dir;
}

// A single raster thread keeps two builds identical to the pixel.
const launch = () => chromium.launch({ args: ['--num-raster-threads=1'] });

async function render(browser, html, file, { width, height, scale = 1, transparent = false }) {
  const page = await browser.newPage({ viewport: { width, height }, deviceScaleFactor: scale });
  await page.setContent(html, { waitUntil: 'load' });
  await page.evaluate(() => Promise.all([...document.fonts].map(face => face.load())));
  await page.evaluate(() => Promise.all([...document.images].map(img => img.decode())));
  const png = await page.screenshot({ path: file, omitBackground: transparent });
  await page.close();
  return png;
}

module.exports = { fonts, logo, komaCheckout, launch, render };
