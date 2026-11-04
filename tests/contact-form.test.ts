import { describe, expect, it } from 'vitest';
import { submitContactForm } from '../src/contact-form';

const ENDPOINT = 'https://formspree.io/f/test123';

function formData() {
  const data = new FormData();
  data.set('ten', 'Nguyễn Văn A');
  data.set('cong_ty', 'Công ty ABC');
  return data;
}

/** A fake `fetch` that records the request and answers with `status` and a JSON `body`. */
function fakeFetch(status: number, body: unknown = {}) {
  const calls: { url: string; init: RequestInit }[] = [];
  const fetchFn = async (url: string | URL | Request, init: RequestInit = {}) => {
    calls.push({ url: String(url), init });
    return new Response(JSON.stringify(body), { status, headers: { 'Content-Type': 'application/json' } });
  };
  return { fetchFn: fetchFn as typeof fetch, calls };
}

describe('gửi form liên hệ doanh nghiệp', () => {
  it('gửi POST nội dung form tới dịch vụ form, xin trả lời dạng JSON', async () => {
    const { fetchFn, calls } = fakeFetch(200, { ok: true });
    const data = formData();
    await submitContactForm(ENDPOINT, data, fetchFn);

    expect(calls).toHaveLength(1);
    expect(calls[0].url).toBe(ENDPOINT);
    expect(calls[0].init.method).toBe('POST');
    expect(new Headers(calls[0].init.headers).get('Accept')).toBe('application/json');
    expect(calls[0].init.body).toBe(data);
  });

  it('báo gửi thành công bằng tiếng Việt', async () => {
    const result = await submitContactForm(ENDPOINT, formData(), fakeFetch(200, { ok: true }).fetchFn);
    expect(result.ok).toBe(true);
    expect(result.message).toContain('Cảm ơn');
  });

  it('báo lỗi bằng tiếng Việt khi dịch vụ từ chối dữ liệu, không lộ thông báo tiếng Anh', async () => {
    const { fetchFn } = fakeFetch(422, { errors: [{ field: 'lien_he', message: 'should be an email' }] });
    const result = await submitContactForm(ENDPOINT, formData(), fetchFn);
    expect(result.ok).toBe(false);
    expect(result.message).toContain('Chưa gửi được');
    expect(result.message).not.toContain('should be');
  });

  it.each([[403], [404]])('không đổ lỗi cho khách khi dịch vụ form từ chối vì cấu hình (HTTP %i)', async (status) => {
    const result = await submitContactForm(ENDPOINT, formData(), fakeFetch(status, { error: 'Form not found' }).fetchFn);
    expect(result.ok).toBe(false);
    expect(result.message).toContain('Chưa gửi được');
    expect(result.message).not.toContain('kiểm tra lại thông tin');
    expect(result.message).toContain('chat');
  });

  it('báo lỗi khi dịch vụ gặp sự cố', async () => {
    const result = await submitContactForm(ENDPOINT, formData(), fakeFetch(500).fetchFn);
    expect(result.ok).toBe(false);
    expect(result.message).toContain('Chưa gửi được');
  });

  it('báo lỗi kết nối khi mất mạng, gợi ý cách liên hệ khác', async () => {
    const offline = (async () => {
      throw new TypeError('Failed to fetch');
    }) as typeof fetch;
    const result = await submitContactForm(ENDPOINT, formData(), offline);
    expect(result.ok).toBe(false);
    expect(result.message).toContain('kết nối');
    expect(result.message).toContain('chat');
  });
});
