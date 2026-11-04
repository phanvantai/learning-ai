import { describe, expect, it } from 'vitest';
import Home from '../src/pages/index.astro';
import { mainContent, renderPage } from './render';

describe('Trang chủ', () => {
  it('nêu định vị "kỹ sư phần mềm dùng AI mỗi ngày"', async () => {
    const html = mainContent(await renderPage(Home));
    expect(html).toContain('Kỹ sư phần mềm dùng AI mỗi ngày');
  });

  it('không tự xưng "chuyên gia AI"', async () => {
    const html = mainContent(await renderPage(Home));
    expect(html.toLowerCase()).not.toContain('chuyên gia ai');
  });
});
