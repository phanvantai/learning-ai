import { describe, expect, it } from 'vitest';
import DoanhNghiep from '../src/pages/doanh-nghiep.astro';
import { mainContent, renderPage } from './render';

/** Visible text of `html` with tags removed and whitespace collapsed. */
function text(html: string): string {
  return html
    .replace(/<script[\s\S]*?<\/script>/g, ' ')
    .replace(/<[^>]+>/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
}

async function pageText() {
  return text(mainContent(await renderPage(DoanhNghiep, '/doanh-nghiep/')));
}

describe('trang Doanh nghiệp', () => {
  it('là trang thật, không còn "Sắp ra mắt"', async () => {
    const page = await pageText();
    expect(page).toContain('Khóa đào tạo');
    expect(page).not.toContain('Sắp ra mắt');
  });

  it('nói rõ đang nhận doanh nghiệp thí điểm', async () => {
    expect((await pageText()).toLowerCase()).toContain('đang nhận doanh nghiệp thí điểm');
  });

  it.each([
    ['Lv0', 'AI là gì, sử dụng cơ bản', 'nửa ngày'],
    ['Lv1', 'Tối ưu công việc cá nhân', 'một ngày'],
    ['Lv2', 'Đưa AI vào quy trình doanh nghiệp', 'nhiều buổi'],
  ])('mô tả Cấp độ %s (%s) với thời lượng tham khảo %s', async (level, summary, duration) => {
    const page = await pageText();
    expect(page).toMatch(new RegExp(`${level}\\b[^]*?${summary}[^]*?Thời lượng tham khảo: ${duration}`));
  });

  it('nói thật rằng Lv2 chỉ mở sau khi đã làm thật một lần', async () => {
    expect(await pageText()).toMatch(/Lv2[^.]*chỉ mở sau khi[^.]*quy trình doanh nghiệp thật/);
  });

  it('nêu hình thức: trực tiếp tại doanh nghiệp hoặc online', async () => {
    expect(await pageText()).toContain('trực tiếp tại doanh nghiệp hoặc online');
  });

  it('không hiển thị giá ở giai đoạn thí điểm', async () => {
    const page = (await pageText()).toLowerCase();
    for (const priceWord of ['đồng', 'vnđ', 'vnd', '₫', 'triệu']) expect(page, priceWord).not.toContain(priceWord);
  });

  it('không dùng từ cần tránh cho Khóa đào tạo, Cấp độ và Khách hàng doanh nghiệp', async () => {
    const page = (await pageText()).toLowerCase();
    for (const term of ['khóa học', 'workshop', 'training', 'level', 'module', 'client', 'công ty đối tác']) {
      expect(page, term).not.toContain(term);
    }
  });
});
