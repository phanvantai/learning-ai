import { getCollection } from 'astro:content';
import { describe, expect, it } from 'vitest';
import ChiTiet from '../src/pages/bai-chia-se/[id].astro';
import DanhSach from '../src/pages/bai-chia-se/index.astro';
import { mainContent, renderPage } from './render';

const tatCaBai = await getCollection('baiChiaSe');

describe('trang danh sách Bài chia sẻ', () => {
  it('liệt kê mọi Bài chia sẻ trong repo, mỗi bài dẫn tới trang chi tiết của nó', async () => {
    const html = mainContent(await renderPage(DanhSach, '/bai-chia-se/'));
    expect(tatCaBai.length).toBeGreaterThan(0);
    for (const bai of tatCaBai) {
      expect(html).toContain(bai.data.title);
      expect(html).toContain(`href="/bai-chia-se/${bai.id}/"`);
    }
  });
});

describe('trang chi tiết Bài chia sẻ', () => {
  it('hiện tiêu đề và nội dung Markdown của bài', async () => {
    const html = mainContent(
      await renderPage(ChiTiet, '/bai-chia-se/tom-tat-email-dai-bang-ai/', { id: 'tom-tat-email-dai-bang-ai' }),
    );
    expect(html).toMatch(/<h1[^>]*>Dùng AI tóm tắt email dài trong 2 phút/);
    expect(html).toMatch(/<h2[^>]*>Câu lệnh mẫu<\/h2>/);
  });
});
