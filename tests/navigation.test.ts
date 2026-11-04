import { describe, expect, it } from 'vitest';
import Mentoring from '../src/pages/mentoring.astro';
import Home from '../src/pages/index.astro';
import ChiTietBaiChiaSe from '../src/pages/bai-chia-se/[id].astro';
import { renderPage } from './render';

/** All `<a>` links in the main navigation, as `{ label, href, current }` (current page or section). */
function navLinks(html: string) {
  const nav = html.match(/<nav[^>]*aria-label="Điều hướng chính"[^>]*>([\s\S]*?)<\/nav>/)?.[1] ?? '';
  return [...nav.matchAll(/<a([^>]*)>\s*([^<]*?)\s*<\/a>/g)].map(([, attrs, label]) => ({
    label,
    href: attrs.match(/href="([^"]*)"/)?.[1],
    current: /aria-current="(page|true)"/.test(attrs),
  }));
}

describe('thanh điều hướng', () => {
  it('có đủ 5 mục điều hướng tới các trang chính', async () => {
    const links = navLinks(await renderPage(Home));
    expect(links.map(({ label, href }) => [label, href])).toEqual([
      ['Trang chủ', '/'],
      ['Bài chia sẻ', '/bai-chia-se/'],
      ['Mentoring', '/mentoring/'],
      ['Doanh nghiệp', '/doanh-nghiep/'],
      ['Giới thiệu', '/gioi-thieu/'],
    ]);
  });

  it('đánh dấu mục của trang đang xem', async () => {
    const links = navLinks(await renderPage(Mentoring, '/mentoring/'));
    expect(links.filter((l) => l.current).map((l) => l.label)).toEqual(['Mentoring']);
  });

  it('trang con của Bài chia sẻ vẫn đánh dấu mục "Bài chia sẻ"', async () => {
    const id = 'tom-tat-email-dai-bang-ai';
    const links = navLinks(await renderPage(ChiTietBaiChiaSe, `/bai-chia-se/${id}/`, { id }));
    expect(links.filter((l) => l.current).map((l) => l.label)).toEqual(['Bài chia sẻ']);
  });
});
