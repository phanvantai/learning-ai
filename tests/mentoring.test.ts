import { describe, expect, it } from 'vitest';
import Mentoring from '../src/pages/mentoring.astro';
import { mainContent, renderPage } from './render';

let page: Promise<string> | undefined;

/** Nội dung chính của trang Mentoring, render một lần cho cả file. */
function mentoringMain() {
  page ??= renderPage(Mentoring, '/mentoring/').then(mainContent);
  return page;
}

/** Chữ người xem thấy trong một đoạn HTML (bỏ thẻ, gộp khoảng trắng). */
function toText(html: string) {
  return html
    .replace(/<[^>]*>/g, ' ')
    .replace(/&times;/g, '×')
    .replace(/\s+/g, ' ');
}

/** Chữ người xem thấy trong nội dung chính của trang Mentoring. */
async function visibleText() {
  return toText(await mentoringMain());
}

/** Chữ trong mục có tiêu đề `<h2>` chứa `heading`, tới trước `<h2>` kế tiếp. */
async function section(heading: string) {
  const parts = (await mentoringMain()).split(/(?=<h2[\s>])/);
  const part = parts.find((p) => p.match(/^<h2[^>]*>([\s\S]*?)<\/h2>/)?.[1].includes(heading));
  return part === undefined ? undefined : toText(part);
}

describe('trang Mentoring: gói beta', () => {
  it('nêu gói 4 buổi × 60 phút trong 1 tháng', async () => {
    expect(await visibleText()).toMatch(/4 buổi × 60 phút trong 1 tháng/);
  });

  it('công khai giá beta 1 triệu đồng mỗi gói', async () => {
    expect(await visibleText()).toMatch(/1 triệu đồng/);
  });

  it('giới hạn 5 suất', async () => {
    expect(await visibleText()).toMatch(/5 suất/);
  });

  it('kết quả là một tool, automation hoặc skill AI chạy được cho công việc của học viên', async () => {
    expect(await visibleText()).toMatch(/tool, automation hoặc skill AI chạy được/);
  });

  it('ghi rõ học viên beta cam kết cho viết case study và để lại lời nhận xét', async () => {
    const text = await visibleText();
    expect(text).toMatch(/cam kết cho (mình )?viết case study và để lại lời nhận xét/);
  });

  it('thanh toán bằng chuyển khoản, không thanh toán online', async () => {
    const text = await visibleText();
    expect(text).toMatch(/chuyển khoản/);
    expect(text).toMatch(/không thanh toán online/);
  });

  it('buổi lẻ chỉ bán thêm cho học viên đã mua gói, không có buổi thử riêng', async () => {
    const text = await visibleText();
    expect(text).toMatch(/[Bb]uổi lẻ chỉ bán thêm cho học viên đã mua gói/);
    expect(text).toMatch(/không có buổi thử riêng/);
  });
});

describe('trang Mentoring: quy trình', () => {
  it('bắt đầu bằng Tư vấn qua chat miễn phí, rồi mới chuyển sang Mentoring', async () => {
    const steps = await section('Quy trình');
    expect(steps).toBeDefined();
    const chat = steps!.indexOf('Tư vấn qua chat');
    const mentoring = steps!.indexOf('Mentoring', chat);
    expect(chat).toBeGreaterThanOrEqual(0);
    expect(mentoring).toBeGreaterThan(chat);
    expect(steps).toMatch(/Tư vấn qua chat[^.]*miễn phí/);
  });

  it('chuyển sang Mentoring khi cần gọi riêng, chia sẻ màn hình hoặc build hộ', async () => {
    expect(await section('Quy trình')).toMatch(/gọi riêng, chia sẻ màn hình hoặc build hộ/);
  });

  it('nói Tư vấn qua chat chính là bước thử trước khi mua Mentoring', async () => {
    expect(await section('Quy trình')).toMatch(/Tư vấn qua chat chính là bước thử/);
  });
});

describe('trang Mentoring: case study', () => {
  it('có khu case study, báo là chưa có khi chưa có case study nào', async () => {
    const caseStudies = await section('Case study');
    expect(caseStudies).toBeDefined();
    expect(caseStudies).toMatch(/[Cc]hưa có case study/);
  });
});

describe('trang Mentoring: chương trình 4 buổi', () => {
  it('mô tả đủ 4 buổi theo thứ tự, mỗi buổi có sản phẩm', async () => {
    const chuongTrinh = await section('Chương trình');
    expect(chuongTrinh).toBeDefined();
    const viTri = [1, 2, 3, 4].map((so) => chuongTrinh!.indexOf(`Buổi ${so} ·`));
    expect(viTri.every((v) => v >= 0)).toBe(true);
    expect([...viTri].sort((a, b) => a - b)).toEqual(viTri);
    expect(chuongTrinh!.match(/Sản phẩm/g)).toHaveLength(4);
  });

  it('nêu ba hướng Skill, Automation, Tool', async () => {
    const huong = await section('Ba hướng');
    for (const ten of ['Skill', 'Automation', 'Tool']) expect(huong, ten).toContain(ten);
  });

  it('nói rõ ai chưa hợp với Mentoring', async () => {
    expect(await section('Hợp và chưa hợp')).toMatch(/Chưa hợp khi/);
  });

  it('ví dụ đề bài chỉ dẫn tới Bài chia sẻ có thật', async () => {
    const { getCollection } = await import('astro:content');
    const ids = new Set((await getCollection('baiChiaSe')).map((bai) => bai.id));
    const links = [...(await mentoringMain()).matchAll(/href="\/bai-chia-se\/([^/"]+)\/"/g)].map((m) => m[1]);
    expect(links.length).toBeGreaterThan(0);
    for (const id of links) expect(ids.has(id), id).toBe(true);
  });
});
