import { getCollection } from 'astro:content';
import { describe, expect, it } from 'vitest';
import { chuDeSlug, tatCaChuDe } from '../src/bai-chia-se';
import TheoChuDe from '../src/pages/bai-chia-se/chu-de/[slug].astro';
import DanhSach from '../src/pages/bai-chia-se/index.astro';
import { mainContent, renderPage } from './render';

const tatCaBai = await getCollection('baiChiaSe');

/** Đường dẫn tới các trang chi tiết Bài chia sẻ, theo thứ tự xuất hiện trong HTML. */
function linksToBai(html: string): string[] {
  return [...html.matchAll(/href="\/bai-chia-se\/([^/"]+)\/"/g)].map(([, id]) => id);
}

/** Các link trong bộ lọc theo thẻ chủ đề, dạng `{ label, href, current }`. */
function filterLinks(html: string) {
  const nav = html.match(/<nav[^>]*aria-label="Lọc theo thẻ chủ đề"[^>]*>([\s\S]*?)<\/nav>/)?.[1] ?? '';
  return [...nav.matchAll(/<a([^>]*)>\s*([^<]*?)\s*<\/a>/g)].map(([, attrs, label]) => ({
    label,
    href: attrs.match(/href="([^"]*)"/)?.[1],
    current: /aria-current="page"/.test(attrs),
  }));
}

const renderChuDe = async (slug: string) =>
  mainContent(await renderPage(TheoChuDe, `/bai-chia-se/chu-de/${slug}/`, { slug }));

describe('lọc Bài chia sẻ theo thẻ chủ đề', () => {
  it('trang danh sách có bộ lọc gồm "Tất cả" và mọi thẻ chủ đề đang dùng', async () => {
    const links = filterLinks(mainContent(await renderPage(DanhSach, '/bai-chia-se/')));
    expect(links[0]).toEqual({ label: 'Tất cả', href: '/bai-chia-se/', current: true });
    const slugTrongBai = new Set(tatCaBai.flatMap((bai) => bai.data.tags.map(chuDeSlug)));
    expect(links.slice(1).map((l) => l.href).sort()).toEqual(
      [...slugTrongBai].map((slug) => `/bai-chia-se/chu-de/${slug}/`).sort(),
    );
  });

  it('mỗi trang thẻ chủ đề chỉ liệt kê các bài có thẻ đó, mới nhất trước', async () => {
    for (const slug of new Set(tatCaBai.flatMap((bai) => bai.data.tags.map(chuDeSlug)))) {
      const html = await renderChuDe(slug);
      const coThe = tatCaBai
        .filter((bai) => bai.data.tags.map(chuDeSlug).includes(slug))
        .sort((a, b) => b.data.date.getTime() - a.data.date.getTime());
      expect(linksToBai(html), slug).toEqual(coThe.map((bai) => bai.id));
    }
  });

  it('trang thẻ chủ đề nêu tên thẻ và đánh dấu thẻ đang chọn trong bộ lọc', async () => {
    const html = await renderChuDe('prompt-mau');
    expect(html).toMatch(/<h1[^>]*>[^<]*Prompt mẫu/);
    expect(filterLinks(html).filter((l) => l.current)).toEqual([
      { label: 'Prompt mẫu', href: '/bai-chia-se/chu-de/prompt-mau/', current: true },
    ]);
  });

  it('mỗi bài hiện thẻ chủ đề của nó, bấm vào là tới trang thẻ đó', async () => {
    const html = mainContent(await renderPage(DanhSach, '/bai-chia-se/'));
    const card = html.split('<article').find((c) => c.includes('href="/bai-chia-se/tom-tat-email-dai-bang-ai/"'));
    expect(card).toMatch(/<a[^>]*href="\/bai-chia-se\/chu-de\/dan-van-phong\/"[^>]*>Dân văn phòng<\/a>/);
  });
});

describe('chuDeSlug', () => {
  it.each([
    ['Email', 'email'],
    ['Prompt mẫu', 'prompt-mau'],
    ['Dân văn phòng', 'dan-van-phong'],
    ['Đọc nhanh', 'doc-nhanh'],
    ['  AI & Excel ', 'ai-excel'],
  ])('"%s" → "%s"', (ten, slug) => {
    expect(chuDeSlug(ten)).toBe(slug);
  });
});

describe('tatCaChuDe', () => {
  const bai = (tags: string[]) => ({ data: { tags } });

  it('gộp thẻ trùng nhau (kể cả khác hoa thường hay dấu), giữ cách viết gặp đầu tiên, xếp theo ABC', () => {
    const chuDe = tatCaChuDe([bai(['Prompt mẫu', 'Email']), bai(['email', 'Ảnh']), bai(['Prompt mau'])]);
    expect(chuDe).toEqual([
      { ten: 'Ảnh', slug: 'anh' },
      { ten: 'Email', slug: 'email' },
      { ten: 'Prompt mẫu', slug: 'prompt-mau' },
    ]);
  });
});
