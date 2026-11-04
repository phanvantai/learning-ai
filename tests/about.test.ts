import { describe, expect, it } from 'vitest';
import GioiThieu from '../src/pages/gioi-thieu.astro';
import { mainContent, renderPage } from './render';

describe('trang Giới thiệu', () => {
  it('giới thiệu người sáng lập là kỹ sư phần mềm, học công khai', async () => {
    const html = mainContent(await renderPage(GioiThieu));
    expect(html).toContain('Giới thiệu');
    expect(html).toContain('kỹ sư phần mềm');
    expect(html).toContain('học công khai');
  });

  it('đánh dấu rõ chỗ người sáng lập cần điền nội dung thật', async () => {
    const html = mainContent(await renderPage(GioiThieu));
    expect(html).toContain('TODO(owner)');
  });

  it('không tự xưng "chuyên gia AI"', async () => {
    const html = mainContent(await renderPage(GioiThieu));
    expect(html.toLowerCase()).not.toContain('chuyên gia ai');
  });
});
