import { describe, expect, it } from 'vitest';
import Home from '../src/pages/index.astro';
import { renderPage } from './render';

describe('thanh điều hướng', () => {
  it('trang chủ có đủ 5 mục điều hướng tới các trang chính', async () => {
    const html = await renderPage(Home);
    for (const [label, href] of [
      ['Trang chủ', '/'],
      ['Bài chia sẻ', '/bai-chia-se/'],
      ['Mentoring', '/mentoring/'],
      ['Doanh nghiệp', '/doanh-nghiep/'],
      ['Giới thiệu', '/gioi-thieu/'],
    ]) {
      expect(html).toMatch(new RegExp(`<a[^>]*href="${href}"[^>]*>\\s*${label}\\s*</a>`));
    }
  });
});
