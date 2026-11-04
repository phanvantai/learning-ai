import { describe, expect, it } from 'vitest';
import BaiChiaSe from '../src/pages/bai-chia-se/index.astro';
import DoanhNghiep from '../src/pages/doanh-nghiep.astro';
import GioiThieu from '../src/pages/gioi-thieu.astro';
import Home from '../src/pages/index.astro';
import Mentoring from '../src/pages/mentoring.astro';
import { mainContent, renderPage } from './render';

/** Từ CONTEXT.md đánh dấu _Avoid_, không được xuất hiện trong nội dung hiển thị. */
const AVOIDED_TERMS = ['hỗ trợ 1:1', 'kèm cặp', 'coaching', 'buổi tư vấn', 'tư vấn miễn phí', 'khóa học'];

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
});
