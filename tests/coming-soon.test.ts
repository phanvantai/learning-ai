import { describe, expect, it } from 'vitest';
import BaiChiaSe from '../src/pages/bai-chia-se/index.astro';
import { mainContent, renderPage } from './render';

describe('trang chưa làm', () => {
  it.each([['Bài chia sẻ', BaiChiaSe]])('trang %s hiện "Sắp ra mắt" thay vì lỗi', async (title, page) => {
    const html = mainContent(await renderPage(page));
    expect(html).toContain(title);
    expect(html).toContain('Sắp ra mắt');
  });
});
