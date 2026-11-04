import { describe, expect, it } from 'vitest';
import BaiChiaSe from '../src/pages/bai-chia-se/index.astro';
import DoanhNghiep from '../src/pages/doanh-nghiep.astro';
import { mainContent, renderPage } from './render';

describe('trang chưa làm', () => {
  it.each([
    ['Bài chia sẻ', BaiChiaSe],
    ['Doanh nghiệp', DoanhNghiep],
  ])('trang %s hiện "Sắp ra mắt" thay vì lỗi', async (title, page) => {
    const html = mainContent(await renderPage(page));
    expect(html).toContain(title);
    expect(html).toContain('Sắp ra mắt');
  });

  it('trang Doanh nghiệp nói rõ đang nhận thí điểm', async () => {
    const html = mainContent(await renderPage(DoanhNghiep));
    expect(html).toContain('đang nhận doanh nghiệp thí điểm');
  });
});
