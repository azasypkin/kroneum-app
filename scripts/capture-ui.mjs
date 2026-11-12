import { chromium } from '@playwright/test';
import { mkdirSync, readFileSync } from 'node:fs';

const output = new URL('../docs/images/', import.meta.url);
mkdirSync(output, { recursive: true });
const browser = await chromium.launch();
try {
  for (const colorScheme of ['light', 'dark']) {
    const page = await browser.newPage({
      viewport: { width: 392, height: 684 },
      deviceScaleFactor: 2,
      colorScheme,
      reducedMotion: 'reduce',
    });
    await page.goto('http://127.0.0.1:1420/?preview=work');
    await page
      .getByRole('button', { name: 'Pause timer', exact: true })
      .waitFor();
    await page
      .locator('.shell')
      .screenshot({ path: new URL(`${colorScheme}.png`, output).pathname });
    await page.goto('http://127.0.0.1:1420/?preview=break-alert');
    await page.getByRole('button', { name: 'Snooze for 5 min' }).hover();
    await page.locator('.shell').screenshot({
      path: new URL(`completion-${colorScheme}.png`, output).pathname,
      animations: 'disabled',
    });
    await page.getByRole('button', { name: 'Open settings' }).click();
    await page.getByRole('switch', { name: /Continue the cycle/ }).check();
    await page.getByRole('button', { name: 'Save settings' }).click();
    await page.getByRole('button', { name: 'Back to timer' }).click();
    await page.getByRole('button', { name: 'Snooze for 5 min' }).hover();
    await page.locator('.timer-actions').screenshot({
      path: new URL(`completion-actions-${colorScheme}.png`, output).pathname,
      animations: 'disabled',
    });
    await page.goto('http://127.0.0.1:1420/?preview=alert');
    await page.getByRole('button', { name: 'Snooze for 5 min' }).click();
    await page.locator('.shell').screenshot({
      path: new URL(`snooze-${colorScheme}.png`, output).pathname,
      animations: 'disabled',
    });
    await page.locator('.timer-face').screenshot({
      path: new URL(`snooze-timer-${colorScheme}.png`, output).pathname,
      animations: 'disabled',
    });
    await page.close();
  }
  const image = (name) =>
    `data:image/svg+xml;base64,${readFileSync(new URL(`../public/brand/${name}.svg`, import.meta.url)).toString('base64')}`;
  const board = await browser.newPage({
    viewport: { width: 900, height: 520 },
    deviceScaleFactor: 2,
  });
  await board.setContent(`<!doctype html><html lang="en"><head><meta charset="utf-8"/><title>Kroneum brand artwork</title><style>
    *{box-sizing:border-box}html{min-height:100%;background:#0e1917}body{min-height:100vh;margin:0;padding:40px;color:#eef3e8;font-family:-apple-system,BlinkMacSystemFont,sans-serif;background:radial-gradient(ellipse at 15% 0%,#284936,transparent 65%),#0e1917}
    .board{display:grid;grid-template-columns:320px 1fr;gap:46px;align-items:center}h1{font-size:36px;font-weight:600;letter-spacing:-1px;margin:0 0 12px}p{color:#becbb7;line-height:1.6;font-size:14px;margin:0 0 28px}
    .app{display:block;width:320px;height:320px}.row{display:flex;align-items:center;gap:20px;margin-block:16px}.mark{width:128px;height:96px}.small{width:40px;height:30px}
    .menu{display:flex;align-items:center;gap:9px;padding:13px 18px;border:1px solid #53634e;border-radius:12px;background:#1f3026;font-size:18px;font-variant-numeric:tabular-nums}.template{width:24px;height:18px;filter:invert(1)}
    .caption{font-size:11px;letter-spacing:1.7px;text-transform:uppercase;color:#b8c5b0}.sizes{display:flex;align-items:center;gap:18px}.sizes img{display:block}
    .note{border-top:1px solid #354c3c;padding-top:22px;margin-top:24px;font-size:12px;color:#b8c5b0}.light{background:#f4f7ed;padding:8px 14px;border-radius:9px}
    </style></head><body><main class="board"><section><img class="app" src="${image('app-icon')}" alt="Kroneum app icon"/><div class="caption" style="text-align:center">macOS · vector master to 1024 px</div></section><section><h1>Kroneum</h1><p>Eye care. Clear focus. A regular rhythm.</p><div class="row"><img class="mark" src="${image('mark')}" alt="Eye, focus spark, and cycle logo"/><div class="menu"><img class="template" src="${image('template')}" alt=""/>15:39</div></div><div class="row"><span class="caption">Small, by design</span><div class="sizes"><img class="small" src="${image('mark')}" alt=""/><span class="light"><img class="small" src="${image('mark-light')}" alt=""/></span></div></div><div class="note">One shared geometry. Native menu-bar template.<br/>Distinct light and dark marks. No bitmap enlargement.</div></section></main></body></html>`);
  await board.evaluate(() =>
    Promise.all(Array.from(document.images, (image) => image.decode())),
  );
  await board.screenshot({ path: new URL('brand.png', output).pathname });
  await board.close();
} finally {
  await browser.close();
}
console.log(
  'Saved timer, completion, snooze, and brand previews in docs/images.',
);
