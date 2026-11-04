import { getCollection } from 'astro:content';
import { describe, expect, it } from 'vitest';
import { moiNhatTruoc } from '../src/bai-chia-se';
import ChiTiet from '../src/pages/bai-chia-se/[id].astro';
import DanhSach from '../src/pages/bai-chia-se/index.astro';
import Home from '../src/pages/index.astro';
import { mainContent, renderPage } from './render';

const tatCaBai = await getCollection('baiChiaSe');

/** Đường dẫn tới các trang chi tiết Bài chia sẻ, theo thứ tự xuất hiện trong HTML. */
function linksToBai(html: string): string[] {
  return [...html.matchAll(/href="\/bai-chia-se\/([^/"]+)\/"/g)].map(([, id]) => id);
}

describe('trang danh sách Bài chia sẻ', () => {
  it('liệt kê mọi Bài chia sẻ trong repo, mỗi bài dẫn tới trang chi tiết của nó', async () => {
    const html = mainContent(await renderPage(DanhSach, '/bai-chia-se/'));
    expect(tatCaBai.length).toBeGreaterThan(0);
    for (const bai of tatCaBai) {
      expect(html).toContain(bai.data.title);
      expect(html).toContain(`href="/bai-chia-se/${bai.id}/"`);
    }
  });

  it('xếp bài mới nhất lên đầu', async () => {
    const html = mainContent(await renderPage(DanhSach, '/bai-chia-se/'));
    expect(linksToBai(html)).toEqual(moiNhatTruoc(tatCaBai).map((bai) => bai.id));
  });

  it('hiện ngày đăng và mô tả ngắn của bài', async () => {
    const html = mainContent(await renderPage(DanhSach, '/bai-chia-se/'));
    expect(html).toMatch(/<time datetime="2026-11-04"[^>]*>4 tháng 11, 2026<\/time>/);
    expect(html).toContain('Một câu lệnh mẫu để AI tóm tắt chuỗi email dài');
  });
});

describe('moiNhatTruoc', () => {
  const bai = (id: string, ngay: string) => ({ id, data: { date: new Date(ngay) } });

  it('xếp theo ngày đăng, mới nhất trước, không đổi danh sách gốc', () => {
    const goc = [bai('cu', '2026-10-01'), bai('moi', '2026-11-04'), bai('giua', '2026-10-20')];
    expect(moiNhatTruoc(goc).map((b) => b.id)).toEqual(['moi', 'giua', 'cu']);
    expect(goc.map((b) => b.id)).toEqual(['cu', 'moi', 'giua']);
  });

  it('bài cùng ngày luôn ra cùng một thứ tự', () => {
    const a = bai('a', '2026-11-04');
    const b = bai('b', '2026-11-04');
    expect(moiNhatTruoc([a, b]).map((x) => x.id)).toEqual(moiNhatTruoc([b, a]).map((x) => x.id));
  });
});

describe('trang chi tiết Bài chia sẻ', () => {
  const renderChiTiet = async (id: string) =>
    mainContent(await renderPage(ChiTiet, `/bai-chia-se/${id}/`, { id }));

  it('hiện tiêu đề và nội dung Markdown của bài', async () => {
    const html = await renderChiTiet('tom-tat-email-dai-bang-ai');
    expect(html).toMatch(/<h1[^>]*>Dùng AI tóm tắt email dài trong 2 phút/);
    expect(html).toMatch(/<h2[^>]*>Câu lệnh mẫu<\/h2>/);
  });

  it('hiện ngày đăng', async () => {
    const html = await renderChiTiet('tom-tat-email-dai-bang-ai');
    expect(html).toMatch(/<time datetime="2026-11-04"[^>]*>4 tháng 11, 2026<\/time>/);
  });

  it('hiện thẻ chủ đề của bài, dẫn tới trang thẻ đó', async () => {
    const html = await renderChiTiet('tom-tat-email-dai-bang-ai');
    expect(html).toMatch(/<a[^>]*href="\/bai-chia-se\/chu-de\/prompt-mau\/"[^>]*>Prompt mẫu<\/a>/);
  });
});

describe('Trang chủ', () => {
  it('hiện 3 Bài chia sẻ mới nhất, mới nhất trước', async () => {
    const html = mainContent(await renderPage(Home));
    const moiNhat = moiNhatTruoc(tatCaBai).slice(0, 3);
    expect(html).toMatch(/<h2[^>]*>Bài chia sẻ mới nhất<\/h2>/);
    expect(linksToBai(html)).toEqual(moiNhat.map((bai) => bai.id));
  });
});
