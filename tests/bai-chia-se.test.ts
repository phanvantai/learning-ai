import { getCollection } from 'astro:content';
import { describe, expect, it } from 'vitest';
import { moiNhatTruoc } from '../src/bai-chia-se';
import BaiChiaSeList from '../src/components/BaiChiaSeList.astro';
import ChiTiet from '../src/pages/bai-chia-se/[id].astro';
import DanhSach from '../src/pages/bai-chia-se/index.astro';
import Home from '../src/pages/index.astro';
import { isNewestFirst, listedBai } from './bai-chia-se-html';
import { mainContent, renderComponent, renderPage } from './render';

const tatCaBai = await getCollection('baiChiaSe');
const ngayMoiNhat = tatCaBai.map((bai) => bai.data.date.toISOString().slice(0, 10)).sort().at(-1);

describe('trang danh sách Bài chia sẻ', () => {
  it('liệt kê mọi Bài chia sẻ trong repo, mỗi bài dẫn tới trang chi tiết của nó', async () => {
    const html = mainContent(await renderPage(DanhSach, '/bai-chia-se/'));
    expect(tatCaBai.length).toBeGreaterThan(0);
    expect(listedBai(html).map((bai) => bai.id).sort()).toEqual(tatCaBai.map((bai) => bai.id).sort());
    for (const bai of tatCaBai) expect(html).toContain(bai.data.title);
  });

  it('xếp bài mới nhất lên đầu', async () => {
    const html = mainContent(await renderPage(DanhSach, '/bai-chia-se/'));
    expect(isNewestFirst(listedBai(html).map((bai) => bai.date))).toBe(true);
  });

  it('hiện ngày đăng và mô tả ngắn của bài', async () => {
    const html = mainContent(await renderPage(DanhSach, '/bai-chia-se/'));
    expect(html).toMatch(/<time datetime="2026-11-04"[^>]*>4 tháng 11, 2026<\/time>/);
    expect(html).toContain('Một câu lệnh mẫu để AI tóm tắt chuỗi email dài');
  });
});

describe('danh sách Bài chia sẻ trống', () => {
  it('báo chưa có bài thay vì để trang trắng', async () => {
    const html = await renderComponent(BaiChiaSeList, { tatCaBai: [] });
    expect(html).toContain('Chưa có Bài chia sẻ nào');
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
    expect(html).toMatch(/<h1[^>]*>Dùng AI tóm tắt email dài/);
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
  it('hiện tối đa 3 Bài chia sẻ, bắt đầu từ bài mới nhất', async () => {
    const html = mainContent(await renderPage(Home));
    const listed = listedBai(html);
    expect(html).toMatch(/<h2[^>]*>Bài chia sẻ mới nhất<\/h2>/);
    expect(listed).toHaveLength(Math.min(3, tatCaBai.length));
    expect(listed[0].date).toBe(ngayMoiNhat);
    expect(isNewestFirst(listed.map((bai) => bai.date))).toBe(true);
  });
});
