import { getCollection } from 'astro:content';
import { describe, expect, it } from 'vitest';
import { LOAI_BAI, moiNhatTruoc } from '../src/bai-chia-se';
import BaiChiaSeList from '../src/components/BaiChiaSeList.astro';
import ChiTiet from '../src/pages/bai-chia-se/[id].astro';
import DanhSach from '../src/pages/bai-chia-se/index.astro';
import Home from '../src/pages/index.astro';
import { isNewestFirst, listedBai } from './bai-chia-se-html';
import { mainContent, renderComponent, renderPage } from './render';

const tatCaBai = await getCollection('baiChiaSe');
const ngayMoiNhat = tatCaBai.map((bai) => bai.data.date.toISOString().slice(0, 10)).sort().at(-1);

/** HTML của mục liệt kê một Loại bài (`<section data-loai="...">`) trên một trang. */
function mucCuaLoai(html: string, ma: string): string {
  const muc = html.split(/(?=<section[^>]*data-loai=")/).find((part) => part.startsWith('<section') && part.includes(`data-loai="${ma}"`));
  if (muc === undefined) throw new Error(`Trang không có mục cho Loại bài "${ma}"`);
  return muc.split('</section>')[0];
}

describe('trang danh sách Bài chia sẻ', () => {
  it('liệt kê mọi Bài chia sẻ trong repo, mỗi bài dẫn tới trang chi tiết của nó', async () => {
    const html = mainContent(await renderPage(DanhSach, '/bai-chia-se/'));
    expect(tatCaBai.length).toBeGreaterThan(0);
    expect(listedBai(html).map((bai) => bai.id).sort()).toEqual(tatCaBai.map((bai) => bai.id).sort());
    for (const bai of tatCaBai) expect(html).toContain(bai.data.title);
  });

  it('tách bài theo Loại bài, trong mỗi loại xếp bài mới nhất lên đầu', async () => {
    const html = mainContent(await renderPage(DanhSach, '/bai-chia-se/'));
    for (const loai of LOAI_BAI) {
      const listed = listedBai(mucCuaLoai(html, loai.ma));
      const cuaLoai = tatCaBai.filter((bai) => bai.data.loai === loai.ma);
      expect(listed.map((bai) => bai.id).sort(), loai.ten).toEqual(cuaLoai.map((bai) => bai.id).sort());
      expect(isNewestFirst(listed.map((bai) => bai.date)), loai.ten).toBe(true);
    }
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
  it('mỗi Loại bài hiện tối đa 3 Bài chia sẻ, bắt đầu từ bài mới nhất của loại đó', async () => {
    const html = mainContent(await renderPage(Home));
    expect(html).toMatch(/<h2[^>]*>Bài chia sẻ mới nhất<\/h2>/);
    for (const loai of LOAI_BAI) {
      const listed = listedBai(mucCuaLoai(html, loai.ma));
      const cuaLoai = moiNhatTruoc(tatCaBai.filter((bai) => bai.data.loai === loai.ma));
      expect(listed.map((bai) => bai.id), loai.ten).toEqual(cuaLoai.slice(0, 3).map((bai) => bai.id));
    }
  });
});

describe('trang chi tiết: mục lục và đọc tiếp', () => {
  it('bài nhiều mục có mục lục dẫn tới từng tiêu đề h2', async () => {
    const id = 'tom-tat-email-dai-bang-ai';
    const html = mainContent(await renderPage(ChiTiet, `/bai-chia-se/${id}/`, { id }));
    expect(html).toMatch(/<nav[^>]*aria-label="Mục lục"[\s\S]*?<a href="#câu-lệnh-mẫu"[^>]*>Câu lệnh mẫu<\/a>/);
  });

  it('gợi ý đọc tiếp chỉ gồm bài khác cùng Loại bài', async () => {
    for (const bai of tatCaBai) {
      const html = mainContent(await renderPage(ChiTiet, `/bai-chia-se/${bai.id}/`, { id: bai.id }));
      const goiY = listedBai(html.split('id="doc-tiep"')[1] ?? '').map((khac) => khac.id);
      expect(goiY, bai.id).not.toContain(bai.id);
      for (const id of goiY) {
        expect(tatCaBai.find((khac) => khac.id === id)?.data.loai, `${bai.id} → ${id}`).toBe(bai.data.loai);
      }
    }
  });
});
