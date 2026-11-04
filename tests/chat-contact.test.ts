import { describe, expect, it } from 'vitest';
import Home from '../src/pages/index.astro';
import Mentoring from '../src/pages/mentoring.astro';
import { chatButtons } from './chat';
import { mainContent, renderPage } from './render';

/** Thứ tự kênh Tư vấn qua chat theo `docs/specs/website.md`. */
const CHANNEL_ORDER = ['Zalo', 'Messenger', 'Telegram', 'LinkedIn'];

describe.each([
  ['Trang chủ', Home, '/'],
  ['Mentoring', Mentoring, '/mentoring/'],
])('nút liên hệ chat trên trang %s', (_, page, path) => {
  it('hiện theo thứ tự Zalo, Messenger, Telegram, LinkedIn', async () => {
    const buttons = chatButtons(mainContent(await renderPage(page, path)));
    expect(buttons?.map((b) => b.name)).toEqual(CHANNEL_ORDER);
  });

  it('nút nào cũng mở một link https thật hoặc ghi "sắp có" và không bấm được', async () => {
    const buttons = chatButtons(mainContent(await renderPage(page, path)));
    expect(buttons).toHaveLength(CHANNEL_ORDER.length);
    for (const { name, href, pending } of buttons!) {
      if (pending) expect(href, name).toBeUndefined();
      else expect(href, name).toMatch(/^https:\/\/\S+$/);
    }
  });
});
