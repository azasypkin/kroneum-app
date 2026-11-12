import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';

for (const colorScheme of ['light', 'dark'] as const) {
  for (const deviceScaleFactor of [1, 2]) {
    test(`new brand assets render in ${colorScheme} appearance at ${deviceScaleFactor}×`, async ({
      browser,
    }) => {
      const page = await browser.newPage({
        baseURL: 'http://127.0.0.1:1420',
        viewport: { width: 392, height: 684 },
        colorScheme,
        deviceScaleFactor,
      });
      try {
        const failures: string[] = [];
        page.on('pageerror', (error) => failures.push(error.message));
        await page.goto('/?preview=work');
        await expect(
          page.getByRole('button', { name: 'Pause timer', exact: true }),
        ).toBeVisible();
        const header = page.locator('.brand-mark img');
        const face = page.locator('.face-eye img');
        await expect(header).toHaveAttribute('src', '/brand/app-icon.svg');
        await expect(header).toHaveAttribute('alt', '');
        await expect(face).toHaveAttribute('alt', '');
        await expect
          .poll(() =>
            header.evaluate(
              (node: HTMLImageElement) =>
                node.complete && node.naturalWidth > 0,
            ),
          )
          .toBe(true);
        const mark = colorScheme === 'dark' ? 'mark.svg' : 'mark-light.svg';
        await expect
          .poll(() =>
            face.evaluate(
              (node: HTMLImageElement) =>
                node.complete && node.naturalWidth > 0 && node.currentSrc,
            ),
          )
          .toContain(`/brand/${mark}`);
        await expect
          .poll(() =>
            page
              .locator('#break-preset svg')
              .evaluate((node) => (node as SVGGraphicsElement).getBBox().width),
          )
          .toBeGreaterThan(15);
        await expect(page.locator('link[rel="icon"]')).toHaveAttribute(
          'href',
          '/brand/app-icon.svg',
        );
        await page
          .getByRole('button', { name: 'Eye break 30 sec', exact: true })
          .click();
        await expect(
          page.getByRole('heading', { name: 'Look beyond the screen.' }),
        ).toBeVisible();
        expect(failures).toEqual([]);
      } finally {
        await page.close();
      }
    });
  }
}

test('the capitalized header exposes a native drag region without including its controls', async ({
  page,
}) => {
  await page.goto('/');
  await expect(page.locator('.brand-name')).toHaveText('Kroneum');
  await expect(page.locator('.app-header')).toHaveAttribute(
    'data-tauri-drag-region',
    'deep',
  );
  await expect(page.locator('.header-actions')).toHaveAttribute(
    'data-tauri-drag-region',
    'false',
  );
  await page.getByRole('button', { name: 'Open settings' }).click();
  await expect(
    page.getByRole('heading', { name: 'Your rhythm.' }),
  ).toBeVisible();
});

test('the countdown is optically centered slightly higher inside the ring', async ({
  page,
}) => {
  await page.goto('/?preview=work');
  await expect(
    page.getByRole('button', { name: 'Pause timer', exact: true }),
  ).toBeVisible();
  const offset = await page.evaluate(() => {
    const time = document.getElementById('time')!.getBoundingClientRect();
    const face = document.querySelector('.timer-face')!.getBoundingClientRect();
    return time.y + time.height / 2 - face.y - face.height / 2;
  });
  expect(Math.abs(offset)).toBeLessThanOrEqual(4);
});

for (const fixture of [
  '',
  'work',
  'break',
  'alert',
  'break-alert',
  'idle-alert',
  'snoozed',
]) {
  test(`timer caption fits inside the ring without clipping (${fixture || 'idle'})`, async ({
    page,
  }) => {
    await page.goto(`/?preview=${fixture === 'snoozed' ? 'alert' : fixture}`);
    await expect(page.locator('#primary')).toBeEnabled();
    if (fixture === 'snoozed') {
      await page.getByRole('button', { name: 'Snooze for 5 min' }).click();
      await expect(page.locator('#caption')).toHaveText(
        'UNTIL THE REMINDER RETURNS',
      );
    }
    for (const width of [340, 392]) {
      await page.setViewportSize({ width, height: 684 });
      for (const wrapping of ['balance', 'wrap']) {
        // Standard wrapping covers the Safari 15 fallback as well as balancing.
        await page
          .locator('#caption')
          .evaluate(
            (node, value) => node.style.setProperty('text-wrap', value),
            wrapping,
          );
        const geometry = await page.locator('#caption').evaluate((caption) => {
          const circle = document
            .querySelector('.ring-track')!
            .getBoundingClientRect();
          const range = document.createRange();
          range.selectNodeContents(caption);
          const center = {
            x: circle.x + circle.width / 2,
            y: circle.y + circle.height / 2,
          };
          // Keep the actual glyph rectangles away from the visible ring stroke.
          const safeRadius = circle.width / 2 - 5;
          const inside = Array.from(range.getClientRects()).every((rect) =>
            [
              [rect.left, rect.top],
              [rect.right, rect.top],
              [rect.left, rect.bottom],
              [rect.right, rect.bottom],
            ].every(
              ([x, y]) =>
                Math.hypot(x! - center.x, y! - center.y) <= safeRadius,
            ),
          );
          const style = getComputedStyle(caption);
          return {
            inside,
            overflow: caption.scrollWidth - caption.clientWidth,
            overflowStyle: style.overflow,
            fontSize: style.fontSize,
          };
        });
        expect(
          geometry.inside,
          `${fixture || 'idle'} at ${width}px with ${wrapping}`,
        ).toBe(true);
        expect(geometry.overflow).toBeLessThanOrEqual(1);
        expect(geometry.overflowStyle).toBe('visible');
        expect(geometry.fontSize).toBe('8px');
      }
    }
  });
}

for (const width of [340, 357, 392]) {
  for (const fixture of ['alert', 'break-alert'] as const) {
    test(`completion actions stay separated on hover at ${width}px (${fixture})`, async ({
      page,
    }) => {
      await page.setViewportSize({ width, height: 684 });
      await page.emulateMedia({ colorScheme: 'dark', reducedMotion: 'reduce' });
      await page.goto(`/?preview=${fixture}`);
      await page.getByRole('button', { name: 'Open settings' }).click();
      await page.getByRole('switch', { name: /Continue the cycle/ }).check();
      await page.getByRole('button', { name: 'Save settings' }).click();
      await page.getByRole('button', { name: 'Back to timer' }).click();
      const snooze = page.getByRole('button', { name: 'Snooze for 5 min' });
      await snooze.hover();
      const geometry = await page.evaluate(() => {
        const primary = document
          .getElementById('primary')!
          .getBoundingClientRect();
        const secondary = document
          .getElementById('secondary')!
          .getBoundingClientRect();
        const accept = document
          .getElementById('acknowledge')!
          .getBoundingClientRect();
        const main = document.querySelector('main')!;
        const middleOfGap = document.elementFromPoint(
          primary.x + primary.width / 2,
          (primary.bottom + secondary.top) / 2,
        );
        return {
          firstGap: secondary.top - primary.bottom,
          secondGap: accept.top - secondary.bottom,
          acceptHeight: accept.height,
          bottom: accept.bottom,
          viewportBottom: main.getBoundingClientRect().bottom,
          gapHasButton: !!middleOfGap?.closest('button'),
        };
      });
      expect(geometry.firstGap).toBeGreaterThanOrEqual(8);
      expect(geometry.secondGap).toBeGreaterThanOrEqual(8);
      expect(geometry.acceptHeight).toBeGreaterThanOrEqual(32);
      expect(geometry.bottom).toBeLessThanOrEqual(geometry.viewportBottom);
      expect(geometry.gapHasButton).toBe(false);
      await expect(
        page.getByRole('button', { name: 'Accept & start next timer' }),
      ).toBeVisible();
    });
  }
}

test('starts, pauses, resumes, and switches timers', async ({ page }) => {
  await page.goto('/');
  await page
    .getByRole('button', { name: 'Start work timer', exact: true })
    .click();
  await expect(page.getByText('Time to focus', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Pause timer', exact: true }).click();
  await expect(
    page.getByRole('button', { name: 'Resume timer' }),
  ).toBeVisible();
  await page.getByRole('button', { name: 'Resume timer' }).click();
  await page.getByRole('button', { name: 'Eye break 30 sec' }).click();
  await expect(
    page.getByRole('heading', { name: 'Look beyond the screen.' }),
  ).toBeVisible();
  await page.getByRole('button', { name: 'Stop timer' }).click();
  await expect(
    page.getByText('Ready when you are', { exact: true }),
  ).toBeVisible();
});

test('snooze returns the same finished reminder', async ({ page }) => {
  await page.goto('/?preview=alert');
  await expect(page.locator('#primary')).toBeFocused();
  await page.getByRole('button', { name: 'Snooze for 5 min' }).click();
  await expect(page.getByText('UNTIL THE REMINDER RETURNS')).toBeVisible();
  await page.getByRole('button', { name: 'End snooze now' }).click();
  await expect(page.locator('#primary')).toBeFocused();
  await expect(
    page.getByRole('heading', { name: 'Your eyes deserve a break.' }),
  ).toBeVisible();
  await page
    .getByRole('button', { name: 'Start eye break', exact: true })
    .click();
  await expect(page.getByText('LOOK FAR AWAY & BLINK')).toBeVisible();
});

test('idle reminder requires a timer or snooze', async ({ page }) => {
  await page.goto('/?preview=idle-alert');
  await expect(
    page.getByRole('button', { name: 'Accept & dismiss' }),
  ).not.toBeVisible();
  await page.getByRole('button', { name: 'Eye break 30 sec' }).click();
  await expect(
    page.getByRole('heading', { name: 'Look beyond the screen.' }),
  ).toBeVisible();
});

test('settings persist in the session and do not change an active timer', async ({
  page,
}) => {
  await page.goto('/?preview=work');
  await page.getByRole('button', { name: 'Open settings' }).click();
  await page.getByRole('spinbutton', { name: /Work timer/ }).fill('600');
  await page.getByRole('spinbutton', { name: /Eye break/ }).fill('45');
  await page.getByRole('button', { name: 'Save settings' }).click();
  await expect(page.getByText('Settings saved.')).toBeVisible();
  await page.getByRole('button', { name: 'Back to timer' }).click();
  await expect(page.getByRole('button', { name: 'Work 10 min' })).toBeVisible();
  await expect(page.locator('#time')).toHaveText(/19:5\d|20:00/);
  await page.getByRole('button', { name: 'Eye break 45 sec' }).click();
  await expect(page.locator('#time')).toHaveText(/00:4[45]/);
});

test('keyboard shortcuts and settings focus work', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Open settings' }).click();
  await expect(page.locator('#settings-heading')).toBeFocused();
  await page.keyboard.press('Escape');
  await expect(
    page.getByRole('button', { name: 'Open settings' }),
  ).toBeFocused();
  await page.keyboard.press('Meta+2');
  await expect(
    page.getByRole('heading', { name: 'Look beyond the screen.' }),
  ).toBeVisible();
});

test('the preview does not pretend to play native audio', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByText('UI preview · no native alarms')).toBeVisible();
  await page.getByRole('button', { name: 'Open settings' }).click();
  await page.getByRole('button', { name: 'Try reminder sound' }).click();
  await expect(
    page.getByText('Native sound is available in the macOS app.'),
  ).toBeVisible();
});

test('auto-hide is opt-in, persists, and can be reset without hiding settings', async ({
  page,
}) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Open settings' }).click();
  const toggle = page.getByRole('switch', {
    name: /Hide window after starting a timer/,
  });
  await expect(toggle).not.toBeChecked();
  await toggle.check();
  await page.getByRole('button', { name: 'Save settings' }).click();
  await expect(page.getByText('Settings saved.')).toBeVisible();
  await page.getByRole('button', { name: 'Back to timer' }).click();
  await page.getByRole('button', { name: 'Open settings' }).click();
  await expect(toggle).toBeChecked();
  await page.getByRole('button', { name: 'Use default settings' }).click();
  await expect(toggle).not.toBeChecked();
  await page.getByRole('button', { name: 'Save settings' }).click();
  await expect(page.getByText('Settings saved.')).toBeVisible();
});

for (const fixture of ['alert', 'break-alert'] as const) {
  test(`auto-hide preference keeps completion actions functional (${fixture})`, async ({
    page,
  }) => {
    await page.clock.install();
    await page.goto(`/?preview=${fixture}`);
    await page.getByRole('button', { name: 'Open settings' }).click();
    await page
      .getByRole('switch', { name: /Hide window after starting a timer/ })
      .check();
    await page.getByRole('button', { name: 'Save settings' }).click();
    await page.getByRole('button', { name: 'Back to timer' }).click();
    const action = fixture === 'alert' ? 'Start eye break' : 'Back to work';
    await page.getByRole('button', { name: action, exact: true }).click();
    await expect(
      page.getByRole('button', { name: 'Pause timer', exact: true }),
    ).toBeVisible();
    // The preview deliberately remains visible. Rust tests cover the hide policy.
    await page.clock.fastForward(fixture === 'alert' ? 30_000 : 1_200_000);
    await expect(
      page.getByText('Your timer is complete', { exact: true }),
    ).toBeVisible();
  });
}

test('screen fading is opt-in and the saved fade duration persists', async ({
  page,
}) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Open settings' }).click();
  const toggle = page.getByRole('switch', { name: /Dim screens after work/ });
  const speed = page.getByRole('spinbutton', { name: /Fade duration/ });
  await expect(toggle).not.toBeChecked();
  await expect(speed).toBeDisabled();
  await toggle.check();
  await expect(speed).toBeEnabled();
  await speed.fill('60');
  await page.getByRole('button', { name: 'Save settings' }).click();
  await page.getByRole('button', { name: 'Back to timer' }).click();
  await page.getByRole('button', { name: 'Open settings' }).click();
  await expect(toggle).toBeChecked();
  await expect(speed).toHaveValue('60');
  await expect(
    page.getByRole('button', { name: 'Restore screens', exact: true }),
  ).not.toBeVisible();
});

test('dimming clears at break completion without acceptance', async ({
  page,
}) => {
  await page.clock.install();
  await page.goto('/?preview=alert');
  await page.getByRole('button', { name: 'Open settings' }).click();
  await page.getByRole('switch', { name: /Dim screens after work/ }).check();
  await page.getByRole('button', { name: 'Save settings' }).click();
  const restore = page.getByRole('button', {
    name: 'Restore screens',
    exact: true,
  });
  await expect(restore).toBeVisible();
  await page.getByRole('button', { name: 'Back to timer' }).click();
  await page
    .getByRole('button', { name: 'Start eye break', exact: true })
    .click();
  await expect(restore).toBeVisible();
  await page.clock.fastForward(30_000);
  await expect(restore).not.toBeVisible();
  await expect(
    page.getByText('Your timer is complete', { exact: true }),
  ).toBeVisible();
});

test('snooze retains dimming and Escape restores screens without accepting the alarm', async ({
  page,
}) => {
  await page.goto('/?preview=alert');
  await page.getByRole('button', { name: 'Open settings' }).click();
  await page.getByRole('switch', { name: /Dim screens after work/ }).check();
  await page.getByRole('button', { name: 'Save settings' }).click();
  await page.getByRole('button', { name: 'Back to timer' }).click();
  await page.getByRole('button', { name: 'Snooze for 5 min' }).click();
  await expect(
    page.getByRole('button', { name: 'Restore screens', exact: true }),
  ).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(
    page.getByRole('button', { name: 'Restore screens', exact: true }),
  ).not.toBeVisible();
  await expect(page.getByText('UNTIL THE REMINDER RETURNS')).toBeVisible();
});

test('disabling dimming clears it while keeping an eye break active', async ({
  page,
}) => {
  await page.goto('/?preview=alert');
  await page.getByRole('button', { name: 'Open settings' }).click();
  await page.getByRole('switch', { name: /Dim screens after work/ }).check();
  await page.getByRole('button', { name: 'Save settings' }).click();
  await page.getByRole('button', { name: 'Back to timer' }).click();
  await page
    .getByRole('button', { name: 'Start eye break', exact: true })
    .click();
  await page.getByRole('button', { name: 'Open settings' }).click();
  await page.getByRole('switch', { name: /Dim screens after work/ }).uncheck();
  await page.getByRole('button', { name: 'Save settings' }).click();
  await expect(
    page.getByRole('button', { name: 'Restore screens', exact: true }),
  ).not.toBeVisible();
  await page.getByRole('button', { name: 'Back to timer' }).click();
  await expect(
    page.getByRole('heading', { name: 'Look beyond the screen.' }),
  ).toBeVisible();
});

for (const from of ['snooze', 'break'] as const) {
  test(`starting work restores screens from ${from} without rearming on save`, async ({
    page,
  }) => {
    await page.clock.install();
    await page.goto('/?preview=alert');
    await page.getByRole('button', { name: 'Open settings' }).click();
    await page.getByRole('switch', { name: /Dim screens after work/ }).check();
    await page.getByRole('button', { name: 'Save settings' }).click();
    await page.getByRole('button', { name: 'Back to timer' }).click();
    if (from === 'break') {
      await page
        .getByRole('button', { name: 'Start eye break', exact: true })
        .click();
    } else {
      await page.getByRole('button', { name: 'Snooze for 5 min' }).click();
    }
    const restore = page.getByRole('button', {
      name: 'Restore screens',
      exact: true,
    });
    await expect(restore).toBeVisible();
    await page
      .getByRole('button', { name: 'Work 20 min', exact: true })
      .click();
    await expect(restore).not.toBeVisible();
    await expect(
      page.getByRole('button', { name: 'Pause timer', exact: true }),
    ).toBeVisible();
    await page.getByRole('button', { name: 'Open settings' }).click();
    await page.getByRole('button', { name: 'Save settings' }).click();
    await expect(restore).not.toBeVisible();
    await page.getByRole('button', { name: 'Back to timer' }).click();
    await page.clock.fastForward(20 * 60 * 1000);
    await expect(restore).toBeVisible();
  });
}

test('maximum dimming and the independent break toggle persist with keyboard slider controls', async ({
  page,
}) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Open settings' }).click();
  const maximum = page.getByRole('slider', {
    name: 'Maximum dimming',
    exact: true,
  });
  const always = page.getByRole('switch', {
    name: /Always dim during eye breaks/,
  });
  await expect(maximum).toHaveValue('85');
  await expect(maximum).toBeDisabled();
  await expect(always).not.toBeChecked();
  await always.check();
  await expect(maximum).toBeEnabled();
  await expect(
    page.getByRole('spinbutton', { name: /Fade duration/ }),
  ).toBeEnabled();
  await maximum.press('Home');
  await expect(maximum).toHaveValue('1');
  await maximum.press('End');
  await expect(maximum).toHaveValue('100');
  await expect(maximum).toHaveAttribute('aria-valuetext', '100% dimming');
  await maximum.press('ArrowLeft');
  await expect(page.locator('#max-dim-value')).toHaveText('99%');
  await page.getByRole('button', { name: 'Save settings' }).click();
  await page.getByRole('button', { name: 'Back to timer' }).click();
  await page.getByRole('button', { name: 'Open settings' }).click();
  await expect(always).toBeChecked();
  await expect(maximum).toHaveValue('99');
  await expect(
    page.getByRole('switch', { name: /Dim screens after work/ }),
  ).not.toBeChecked();
});

test('always dim covers a manual eye break and restores automatically at completion', async ({
  page,
}) => {
  await page.clock.install();
  await page.goto('/');
  await page.getByRole('button', { name: 'Open settings' }).click();
  await page
    .getByRole('switch', { name: /Always dim during eye breaks/ })
    .check();
  await page.getByRole('button', { name: 'Save settings' }).click();
  await page.getByRole('button', { name: 'Back to timer' }).click();
  await page
    .getByRole('button', { name: 'Eye break 30 sec', exact: true })
    .click();
  const restore = page.getByRole('button', {
    name: 'Restore screens',
    exact: true,
  });
  await expect(restore).toBeVisible();
  await page.clock.fastForward(30_000);
  await expect(restore).not.toBeVisible();
  await expect(
    page.getByText('Your timer is complete', { exact: true }),
  ).toBeVisible();
});

test('turning off one mode keeps the other mode and its controls active', async ({
  page,
}) => {
  await page.goto('/?preview=alert');
  await page.getByRole('button', { name: 'Open settings' }).click();
  const afterWork = page.getByRole('switch', {
    name: /Dim screens after work/,
  });
  const always = page.getByRole('switch', {
    name: /Always dim during eye breaks/,
  });
  await afterWork.check();
  await always.check();
  await page.getByRole('button', { name: 'Save settings' }).click();
  await page.getByRole('button', { name: 'Back to timer' }).click();
  await page
    .getByRole('button', { name: 'Start eye break', exact: true })
    .click();
  await page.getByRole('button', { name: 'Open settings' }).click();
  await afterWork.uncheck();
  await expect(
    page.getByRole('slider', { name: 'Maximum dimming' }),
  ).toBeEnabled();
  await page.getByRole('button', { name: 'Save settings' }).click();
  await expect(
    page.getByRole('button', { name: 'Restore screens', exact: true }),
  ).toBeVisible();
  await always.uncheck();
  await expect(
    page.getByRole('slider', { name: 'Maximum dimming' }),
  ).toBeDisabled();
  await page.getByRole('button', { name: 'Save settings' }).click();
  await expect(
    page.getByRole('button', { name: 'Restore screens', exact: true }),
  ).not.toBeVisible();
});

test('enabling always dim during an active break restores on Work and does not rearm on save', async ({
  page,
}) => {
  await page.goto('/?preview=break');
  await page.getByRole('button', { name: 'Open settings' }).click();
  await page
    .getByRole('switch', { name: /Always dim during eye breaks/ })
    .check();
  await page.getByRole('button', { name: 'Save settings' }).click();
  const restore = page.getByRole('button', {
    name: 'Restore screens',
    exact: true,
  });
  await expect(restore).toBeVisible();
  await page.getByRole('button', { name: 'Back to timer' }).click();
  await page.getByRole('button', { name: 'Work 20 min', exact: true }).click();
  await expect(restore).not.toBeVisible();
  await page.getByRole('button', { name: 'Open settings' }).click();
  await page.getByRole('button', { name: 'Save settings' }).click();
  await expect(restore).not.toBeVisible();
});

for (const colorScheme of ['light', 'dark'] as const) {
  test(`accessible enabled fade controls in ${colorScheme} appearance`, async ({
    page,
  }) => {
    await page.emulateMedia({ colorScheme });
    await page.goto('/');
    await page.getByRole('button', { name: 'Open settings' }).click();
    await page
      .getByRole('switch', { name: /Always dim during eye breaks/ })
      .check();
    const results = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
      .analyze();
    expect(results.violations).toEqual([]);
  });
  for (const fixture of [
    '',
    'work',
    'break',
    'alert',
    'idle-alert',
    'break-alert',
    'settings',
  ]) {
    test(`accessible ${fixture || 'idle'} view in ${colorScheme} appearance`, async ({
      page,
    }) => {
      await page.emulateMedia({ colorScheme, reducedMotion: 'reduce' });
      await page.goto(`/?preview=${fixture}`);
      await expect(
        page.getByRole('button', {
          name: 'Work 20 min',
          exact: false,
        }),
      ).toBeVisible();
      if (fixture === 'settings')
        await page.getByRole('button', { name: 'Open settings' }).click();
      const results = await new AxeBuilder({ page })
        .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
        .analyze();
      expect(results.violations).toEqual([]);
      expect(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= innerWidth,
        ),
      ).toBe(true);
      if (fixture !== 'settings') {
        expect(
          await page
            .locator('main')
            .evaluate((node) => node.scrollHeight - node.clientHeight),
        ).toBeLessThanOrEqual(1);
      }
    });
  }
}
