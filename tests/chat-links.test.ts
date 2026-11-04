import { describe, expect, it, vi } from 'vitest';
import Home from '../src/pages/index.astro';
import { chatButtons } from './chat';
import { mainContent, renderPage } from './render';

// Giả lập chủ trang đã điền link Zalo, các kênh khác chưa điền.
vi.mock('../src/site', async (importOriginal) => {
  const site = await importOriginal<typeof import('../src/site')>();
  return {
    ...site,
    CHAT_CHANNELS: [
      { name: 'Zalo', href: 'https://zalo.me/0900000000' },
      { name: 'Messenger', href: null },
      { name: 'Telegram', href: null },
      { name: 'LinkedIn', href: null },
    ],
  };
});

describe('link chat cấu hình trong src/site.ts', () => {
  it('kênh đã có link thì là nút bấm mở đúng link đó', async () => {
    const buttons = chatButtons(mainContent(await renderPage(Home)));
    expect(buttons?.find((b) => b.name === 'Zalo')?.href).toBe('https://zalo.me/0900000000');
  });

  it('kênh chưa có link thì vẫn hiện nhưng không bấm được, ghi "sắp có"', async () => {
    const buttons = chatButtons(mainContent(await renderPage(Home)));
    const telegram = buttons?.find((b) => b.name === 'Telegram');
    expect(telegram).toBeDefined();
    expect(telegram?.href).toBeUndefined();
    expect(telegram?.pending).toBe(true);
  });
});
