// node readme/build.cjs <koma checkout> : the app icon, the README hero and feature panels (light/dark), the site's images and the social preview
const fs = require('fs');
const path = require('path');
const { fonts, logo, komaCheckout, launch, render } = require('../lib.cjs');

const koma = komaCheckout(process.argv[2]);
const inputs = path.join(__dirname, 'inputs');
const readme = path.join(koma, '.github/readme');
const site = path.join(koma, 'site/img');
const b64 = file => 'data:image/png;base64,' + fs.readFileSync(path.join(inputs, file)).toString('base64');
const mark = logo(koma);

// The small cut up to 32 px, where the master's gap closes.
const ICON_SIZES = [16, 20, 24, 32, 40, 48, 64, 256];
async function icon(browser) {
  const images = [];
  for (const size of ICON_SIZES) {
    const svg = logo(koma, size <= 32 ? 'logo-small.svg' : 'logo.svg');
    const html = `<style>html, body { margin: 0; background: transparent; } svg { display: block; width: ${size}px; height: ${size}px; }</style>${svg}`;
    images.push(await render(browser, html, undefined, { width: size, height: size, transparent: true }));
  }
  const header = Buffer.alloc(6 + 16 * images.length);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(images.length, 4);
  let offset = header.length;
  images.forEach((png, i) => {
    const entry = 6 + 16 * i;
    // 0 stands for 256.
    header[entry] = header[entry + 1] = ICON_SIZES[i] % 256;
    header.writeUInt16LE(1, entry + 4);
    header.writeUInt16LE(32, entry + 6);
    header.writeUInt32LE(png.length, entry + 8);
    header.writeUInt32LE(offset, entry + 12);
    offset += png.length;
  });
  fs.writeFileSync(path.join(koma, 'windows/koma.ico'), Buffer.concat([header, ...images]));
}

const base = dark => `<!doctype html><meta charset="utf-8">${fonts}
<style>
  :root { --bg: ${dark ? '#171614' : '#f5f3ee'}; --ink: ${dark ? '#edeae3' : '#1c1b18'}; --muted: ${dark ? '#a09b90' : '#615d55'}; --line: ${dark ? '#edeae326' : '#1c1b1822'};
    --accent: ${dark ? '#ff8566' : '#d93a24'}; --key: ${dark ? '#211f1c' : '#fffefb'};
    --wash: ${dark
      ? 'radial-gradient(60% 80% at 15% 10%, #b2381f 0%, transparent 70%), radial-gradient(55% 70% at 90% 25%, #7a2414 0%, transparent 70%), radial-gradient(70% 70% at 60% 110%, #e5573c 0%, transparent 65%), #40150c'
      : 'radial-gradient(60% 80% at 12% 8%, #ffc2b0 0%, transparent 70%), radial-gradient(55% 70% at 92% 20%, #f7856b 0%, transparent 70%), radial-gradient(70% 70% at 55% 115%, #e5432d 0%, transparent 65%), #ffd9cd'}; }
  * { box-sizing: border-box; }
  html, body { margin: 0; background: transparent; }
  body { width: 1280px; overflow: hidden; color: var(--ink); font-family: system-ui, sans-serif; }
  .card { position: relative; width: 1280px; overflow: hidden; border-radius: 28px; background: var(--bg); }
  .serif { font-family: 'Instrument Serif', Georgia, serif; font-weight: 400; letter-spacing: -0.01em; }
  .serif i { color: var(--accent); }
  .stage { position: relative; overflow: hidden; background: var(--wash); }
  /* A window onto a capture: --x --y its top left in the capture's points, --z the zoom. */
  .shot { position: absolute; overflow: hidden; border-radius: 12px; box-shadow: 0 30px 70px -20px #0009, 0 0 0 1px #0000001a; }
  .shot img { position: absolute; left: calc(var(--x) * var(--z) * -1px); top: calc(var(--y) * var(--z) * -1px); width: calc(1600px * var(--z)); }
  .keys { display: flex; gap: 8px; font: 600 15px ui-monospace, 'Cascadia Code', monospace; }
  .keys span { display: grid; place-items: center; min-width: 40px; height: 40px; padding: 0 10px; border: 1px solid var(--line); border-bottom-width: 3px; border-radius: 7px; background: var(--key); }
</style>`;
const shot = (file, x, y, z, style) => `<div class="shot" style="--x:${x};--y:${y};--z:${z};${style}"><img src="${b64(file)}"></div>`;

const hero = (dark, h = 760) => `${base(dark)}<style>
  .card { height: ${h}px; text-align: center; }
  .brand { display: flex; justify-content: center; align-items: center; gap: 12px; padding-top: ${h < 700 ? 34 : 48}px; font-size: 22px; font-weight: 700; letter-spacing: -0.02em; }
  .brand svg { width: 30px; height: 30px; }
  h1 { margin: ${h < 700 ? 18 : 34}px 0 0; font-size: ${h < 700 ? 104 : 124}px; line-height: 0.95; }
  p { margin: 22px auto 0; width: 600px; font-size: 21px; line-height: 1.4; color: var(--muted); }
  .stage { position: absolute; left: 120px; right: 120px; top: ${h < 700 ? 330 : 388}px; height: 700px; border-radius: 24px; }
</style>
<div class="card">
  <div class="brand">${mark}Koma</div>
  <h1 class="serif">your screen, <i>framed</i></h1>
  <p>A screenshot tool for Windows. Freeze the desktop, drag an area, mark it up, then copy or save it.</p>
  <div class="stage">${shot(`marks-${dark ? 'dark' : 'light'}.png`, 70, 50, 0.8, 'left:64px;top:44px;width:912px;height:620px')}</div>
</div>`;

const feature = (dark, n, title, text, extra, stage) => `${base(dark)}<style>
  .card { height: 560px; display: grid; grid-template-columns: 1fr 640px; }
  .copy { padding: 0 56px 0 72px; display: flex; flex-direction: column; justify-content: center; }
  .num { font: 500 15px ui-monospace, 'Cascadia Code', monospace; color: var(--muted); }
  h2 { margin: 14px 0 0; font-size: 64px; line-height: 1.02; }
  p { margin: 20px 0 0; font-size: 19px; line-height: 1.5; color: var(--muted); }
  .extra { margin-top: 30px; }
  .legend { margin-top: 10px; font-size: 14px; color: var(--muted); }
  .stage { margin: 28px 28px 28px 0; border-radius: 20px; }
</style>
<div class="card">
  <div class="copy"><div class="num">${n}</div><h2 class="serif">${title}</h2><p>${text}</p>${extra}</div>
  <div class="stage">${stage}</div>
</div>`;

const keys = (caps, legend) => `<div class="extra"><div class="keys">${caps.map(cap => `<span>${cap}</span>`).join('')}</div><div class="legend">${legend}</div></div>`;
const features = scheme => {
  const dark = scheme === 'dark';
  return {
    'feature-select': feature(dark, '01', 'Freeze, then <i>frame</i>', 'PrtSc freezes every display as it is. Drag the area you want, then nudge its edges until the pixel count is right.',
      keys(['PrtSc'], 'starts a capture from anywhere'),
      shot(`select-${scheme}.png`, 85, 72, 0.85, 'left:48px;top:56px;width:640px;height:500px')),
    'feature-marks': feature(dark, '02', 'Say it <i>on</i> the picture', 'Pen, line, arrow, rectangle, ellipse and text, in six colours and three weights. Undo and redo as far as you like.',
      keys(['P', 'L', 'A', 'R', 'E', 'T'], 'pen · line · arrow · rectangle · ellipse · text'),
      shot(`marks-${scheme}.png`, 600, 290, 1, 'left:48px;top:56px;width:640px;height:500px')),
    'feature-pixelate': feature(dark, '03', 'Hidden <i>for good</i>', 'Pixelate blends each block with what is far around it and adds noise, so nobody can work back to what was there.',
      keys(['X'], 'pixelate'),
      shot(`marks-${scheme}.png`, 148, 575, 1.3, 'left:24px;top:120px;width:564px;height:264px')),
    'feature-deliver': feature(dark, '04', 'Out in <i>one</i> key', 'Enter copies the picture to the clipboard. Ctrl+S saves it to your Screenshots folder. Esc changes your mind.',
      keys(['Enter', 'Ctrl+S', 'Esc'], 'copy · save · cancel'),
      shot(`marks-${scheme}.png`, 548, 706, 1.1, 'left:20px;top:150px;width:572px;height:204px')),
  };
};

// The site's pictures: parts of a capture at its own pixels, in the capture's points.
const SCALE = 1.25;
const CROPS = { hero: ['marks', 96, 68, 1108, 770], select: ['select', 96, 68, 620, 430], marks: ['marks', 590, 280, 560, 400], pixelate: ['marks', 150, 588, 440, 114], deliver: ['marks', 540, 736, 540, 96] };
const crop = (file, x, y) => `<style>html, body { margin: 0; } img { position: absolute; left: ${-x}px; top: ${-y}px; width: 1600px; }</style><img src="${b64(file)}">`;

(async () => {
  const browser = await launch();
  await icon(browser);
  fs.mkdirSync(readme, { recursive: true });
  fs.mkdirSync(site, { recursive: true });
  for (const scheme of ['light', 'dark']) {
    for (const [name, [input, x, y, width, height]] of Object.entries(CROPS)) {
      await render(browser, crop(`${input}-${scheme}.png`, x, y), `${site}/${name}-${scheme}.png`, { width, height, scale: SCALE });
    }
    await render(browser, hero(scheme === 'dark'), `${readme}/hero-${scheme}.png`, { width: 1280, height: 760, scale: 1.6, transparent: true });
    for (const [name, html] of Object.entries(features(scheme))) {
      await render(browser, html, `${readme}/${name}-${scheme}.png`, { width: 1280, height: 560, scale: 1.6, transparent: true });
    }
  }
  // GitHub has no API for the social preview; it is uploaded by hand from this file.
  await render(browser, hero(false, 640), path.join(__dirname, '../social-preview.png'), { width: 1280, height: 640 });
  await browser.close();
})();
