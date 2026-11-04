import { describe, expect, it } from 'vitest';
import NotFound from '../src/pages/404.astro';
import { mainContent, renderPage } from './render';

describe('trang 404', () => {
  it('báo bằng tiếng Việt và dẫn về Trang chủ', async () => {
    const html = mainContent(await renderPage(NotFound, '/khong-ton-tai/'));
    expect(html).toContain('Không tìm thấy trang');
    expect(html).toMatch(/<a[^>]*href="\/"/);
  });
});
