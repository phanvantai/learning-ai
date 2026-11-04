import { getCollection } from 'astro:content';
import { describe, expect, it } from 'vitest';
import { chuDeSlug } from '../src/bai-chia-se';
import ChiTietBaiChiaSe from '../src/pages/bai-chia-se/[id].astro';
import TheoChuDe from '../src/pages/bai-chia-se/chu-de/[slug].astro';
import BaiChiaSe from '../src/pages/bai-chia-se/index.astro';
import DoanhNghiep from '../src/pages/doanh-nghiep.astro';
import GioiThieu from '../src/pages/gioi-thieu.astro';
import Home from '../src/pages/index.astro';
import Mentoring from '../src/pages/mentoring.astro';
import { mainContent, renderPage } from './render';

/** Từ CONTEXT.md đánh dấu _Avoid_, không được xuất hiện trong nội dung hiển thị. */
const AVOIDED_TERMS = ['hỗ trợ 1:1', 'kèm cặp', 'coaching', 'buổi tư vấn', 'tư vấn miễn phí', 'khóa học'];

const tatCaBai = await getCollection('baiChiaSe');

describe('nội dung dùng đúng thuật ngữ trong CONTEXT.md', () => {
  it.each([
    ['Trang chủ', Home],
    ['Bài chia sẻ', BaiChiaSe],
    ['Mentoring', Mentoring],
    ['Doanh nghiệp', DoanhNghiep],
    ['Giới thiệu', GioiThieu],
  ])('trang %s không dùng từ cần tránh', async (_, page) => {
    const html = mainContent(await renderPage(page)).toLowerCase();
    for (const term of AVOIDED_TERMS) expect(html, term).not.toContain(term);
  });

  it.each(tatCaBai.map((bai) => [bai.id]))('Bài chia sẻ "%s" không dùng từ cần tránh', async (id) => {
    const html = mainContent(await renderPage(ChiTietBaiChiaSe, `/bai-chia-se/${id}/`, { id })).toLowerCase();
    for (const term of [...AVOIDED_TERMS, 'chuyên gia ai']) expect(html, term).not.toContain(term);
  });

  it.each([...new Set(tatCaBai.flatMap((bai) => bai.data.tags.map(chuDeSlug)))])(
    'trang thẻ chủ đề "%s" không dùng từ cần tránh',
    async (slug) => {
      const html = mainContent(await renderPage(TheoChuDe, `/bai-chia-se/chu-de/${slug}/`, { slug })).toLowerCase();
      for (const term of AVOIDED_TERMS) expect(html, term).not.toContain(term);
    },
  );
});
