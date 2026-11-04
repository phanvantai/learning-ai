import { describe, expect, it } from 'vitest';
import GioiThieu from '../src/pages/gioi-thieu.astro';
import { mainContent, renderPage } from './render';

describe('trang Giới thiệu', () => {
  it('nói theo tinh thần học công khai', async () => {
    const html = mainContent(await renderPage(GioiThieu));
    expect(html).toContain('học công khai');
  });

  it('nói rõ Mentoring là dịch vụ trả phí, còn Bài chia sẻ và Tư vấn qua chat miễn phí', async () => {
    const html = mainContent(await renderPage(GioiThieu));
    expect(html).toMatch(/Mentoring<\/strong>[^<]*trả phí/);
    expect(html).toMatch(/Bài chia sẻ<\/strong>[^<]*miễn phí/);
    expect(html).toMatch(/Tư vấn qua chat<\/strong>[^<]*miễn phí/);
  });

  it('không tự xưng "chuyên gia AI"', async () => {
    const html = mainContent(await renderPage(GioiThieu));
    expect(html.toLowerCase()).not.toContain('chuyên gia ai');
  });
});
