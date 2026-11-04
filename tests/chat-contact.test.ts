import { experimental_AstroContainer as AstroContainer } from 'astro/container';
import { describe, expect, it } from 'vitest';
import ChatContact from '../src/components/ChatContact.astro';
import Home from '../src/pages/index.astro';
import Mentoring from '../src/pages/mentoring.astro';
import { mainContent, renderPage } from './render';

/** Thứ tự kênh Tư vấn qua chat theo `docs/specs/website.md`. */
const CHANNEL_ORDER = ['Zalo', 'Messenger', 'Telegram', 'LinkedIn'];

/** Các nút trong danh sách kênh Tư vấn qua chat: tên hiển thị, link (nếu có) và chữ hiển thị đầy đủ. */
function chatButtons(html: string) {
  const list = html.match(/<ul[^>]*aria-label="Kênh Tư vấn qua chat"[^>]*>([\s\S]*?)<\/ul>/)?.[1];
  if (list === undefined) return undefined;
  return [...list.matchAll(/<li[^>]*>([\s\S]*?)<\/li>/g)].map(([, item]) => {
    const text = item.replace(/<[^>]*>/g, ' ').replace(/\s+/g, ' ').trim();
    return {
      name: text.split(' ')[0],
      text,
      href: item.match(/<a[^>]*href="([^"]*)"/)?.[1],
    };
  });
}

describe('nút liên hệ chat', () => {
  it.each([
    ['Trang chủ', Home, '/'],
    ['Mentoring', Mentoring, '/mentoring/'],
  ])('trang %s hiện nút chat theo thứ tự Zalo, Messenger, Telegram, LinkedIn', async (_, page, path) => {
    const buttons = chatButtons(mainContent(await renderPage(page, path)));
    expect(buttons?.map((b) => b.name)).toEqual(CHANNEL_ORDER);
  });

  it.each([
    ['Trang chủ', Home, '/'],
    ['Mentoring', Mentoring, '/mentoring/'],
  ])('trang %s không có nút chat nào dẫn tới link hỏng', async (_, page, path) => {
    const buttons = chatButtons(mainContent(await renderPage(page, path))) ?? [];
    for (const { name, href } of buttons) {
      if (href !== undefined) expect(href, name).toMatch(/^https:\/\/[^\s]+$/);
    }
  });

  it('kênh đã có link thì là nút bấm mở link đó', async () => {
    const container = await AstroContainer.create();
    const html = await container.renderToString(ChatContact, {
      props: { title: 'Nhắn tin', channels: [{ name: 'Zalo', href: 'https://zalo.me/0900000000' }] },
    });
    expect(chatButtons(html)).toEqual([
      expect.objectContaining({ name: 'Zalo', href: 'https://zalo.me/0900000000' }),
    ]);
  });

  it('kênh chưa có link thì vẫn hiện nhưng không bấm được, ghi "sắp có"', async () => {
    const container = await AstroContainer.create();
    const html = await container.renderToString(ChatContact, {
      props: { title: 'Nhắn tin', channels: [{ name: 'Telegram', href: null }] },
    });
    const [telegram] = chatButtons(html) ?? [];
    expect(telegram?.name).toBe('Telegram');
    expect(telegram?.href).toBeUndefined();
    expect(telegram?.text).toContain('sắp có');
  });
});
