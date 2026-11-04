import { getCollection } from 'astro:content';
import { describe, expect, it } from 'vitest';
import { chuDeSlug, tatCaChuDe } from '../src/chu-de';
import TheoChuDe from '../src/pages/bai-chia-se/chu-de/[slug].astro';
import DanhSach from '../src/pages/bai-chia-se/index.astro';
import { isNewestFirst, listedBai, slugsInUse } from './bai-chia-se-html';
import { mainContent, navLinks, renderPage } from './render';

const tatCaBai = await getCollection('baiChiaSe');

const filterLinks = (html: string) => navLinks(html, 'Lọc theo thẻ chủ đề');

const renderChuDe = async (slug: string) =>
  mainContent(await renderPage(TheoChuDe, `/bai-chia-se/chu-de/${slug}/`, { slug }));

describe('lọc Bài chia sẻ theo thẻ chủ đề', () => {
  it('trang danh sách có bộ lọc gồm "Tất cả" và mọi thẻ chủ đề đang dùng', async () => {
    const links = filterLinks(mainContent(await renderPage(DanhSach, '/bai-chia-se/')));
    expect(links[0]).toEqual({ label: 'Tất cả', href: '/bai-chia-se/', current: true });
    expect(links.slice(1).map((l) => l.href).sort()).toEqual(
      [...slugsInUse(tatCaBai)].map((slug) => `/bai-chia-se/chu-de/${slug}/`).sort(),
    );
  });

  it('mỗi trang thẻ chủ đề chỉ liệt kê các bài có thẻ đó, mới nhất trước', async () => {
    for (const slug of slugsInUse(tatCaBai)) {
      const listed = listedBai(await renderChuDe(slug));
      const soBaiCoThe = tatCaBai.filter((bai) => bai.data.tags.some((ten) => chuDeSlug(ten) === slug)).length;
      expect(listed, slug).toHaveLength(soBaiCoThe);
      for (const bai of listed) expect(bai.html, `${slug}: ${bai.id}`).toContain(`href="/bai-chia-se/chu-de/${slug}/"`);
      expect(isNewestFirst(listed.map((bai) => bai.date)), slug).toBe(true);
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
    const card = listedBai(html).find((bai) => bai.id === 'tom-tat-email-dai-bang-ai');
    expect(card?.html).toMatch(/<a[^>]*href="\/bai-chia-se\/chu-de\/tom-tat\/"[^>]*>Tóm tắt<\/a>/);
  });
});

describe('chuDeSlug', () => {
  it.each([
    ['Email', 'email'],
    ['Prompt mẫu', 'prompt-mau'],
    ['Dân văn phòng', 'dan-van-phong'],
    ['Đọc nhanh', 'doc-nhanh'],
    ['  AI & Excel ', 'ai-excel'],
    ['🙂', ''],
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
