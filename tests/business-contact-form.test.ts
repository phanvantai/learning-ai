import { describe, expect, it } from 'vitest';
import BusinessContactForm from '../src/components/BusinessContactForm.astro';
import DoanhNghiep from '../src/pages/doanh-nghiep.astro';
import { mainContent, renderComponent, renderPage } from './render';

const ENDPOINT = 'https://formspree.io/f/test123';

/** The opening tag of the form control that `<label>` with text `label` points to (via `for`/`id`). */
function controlLabelled(html: string, label: string): string | undefined {
  const id = html.match(new RegExp(`<label[^>]*for="([^"]+)"[^>]*>\\s*${label}`))?.[1];
  if (!id) return undefined;
  return html.match(new RegExp(`<(?:input|select|textarea)[^>]*id="${id}"[^>]*>`))?.[0];
}

function formTag(html: string): string {
  const tag = html.match(/<form[^>]*>/)?.[0];
  if (!tag) throw new Error('No <form> rendered');
  return tag;
}

describe('form liên hệ trên trang Doanh nghiệp', () => {
  it('có trên trang Doanh nghiệp', async () => {
    const html = mainContent(await renderPage(DoanhNghiep, '/doanh-nghiep/'));
    expect(controlLabelled(html, 'Công ty')).toBeDefined();
  });

  it.each([
    ['Họ tên', 'ten'],
    ['Công ty', 'cong_ty'],
    ['Quy mô', 'quy_mo'],
    ['Nhu cầu', 'nhu_cau'],
    ['Liên hệ', 'lien_he'],
  ])('hỏi "%s" bằng một ô bắt buộc có nhãn', async (label, name) => {
    const html = await renderComponent(BusinessContactForm, { endpoint: ENDPOINT });
    const control = controlLabelled(html, label);
    expect(control, label).toBeDefined();
    expect(control).toContain(`name="${name}"`);
    expect(control).toMatch(/\brequired\b/);
  });

  describe('khi đã cấu hình dịch vụ form', () => {
    it('gửi POST tới dịch vụ form, kể cả khi trình duyệt tắt JavaScript', async () => {
      const form = formTag(await renderComponent(BusinessContactForm, { endpoint: ENDPOINT }));
      expect(form).toContain(`action="${ENDPOINT}"`);
      expect(form).toMatch(/method="post"/i);
    });

    it('có chỗ báo kết quả gửi cho trình đọc màn hình', async () => {
      const html = await renderComponent(BusinessContactForm, { endpoint: ENDPOINT });
      expect(html).toMatch(/<[^>]*role="status"[^>]*>/);
    });

    it('cho phép điền và gửi', async () => {
      const html = await renderComponent(BusinessContactForm, { endpoint: ENDPOINT });
      expect(html).not.toMatch(/<fieldset[^>]*disabled/);
      expect(html).not.toContain('đang được hoàn thiện');
    });
  });

  describe('khi chưa cấu hình dịch vụ form', () => {
    it.each([[''], ['TODO'], ['http://formspree.io/f/insecure']])(
      'với địa chỉ "%s": khóa form và không gửi đi đâu cả',
      async (endpoint) => {
        const html = await renderComponent(BusinessContactForm, { endpoint });
        expect(formTag(html)).not.toContain('action=');
        expect(html).toMatch(/<fieldset[^>]*disabled/);
      },
    );

    it('nói rõ form chưa dùng được và chỉ cách liên hệ qua chat', async () => {
      const html = await renderComponent(BusinessContactForm, { endpoint: '' });
      expect(html).toContain('đang được hoàn thiện');
      expect(html).toMatch(/<a[^>]*href="\/"[^>]*>[^<]*chat/);
    });
  });
});
