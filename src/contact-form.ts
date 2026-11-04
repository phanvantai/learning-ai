/**
 * Form liên hệ cho Khách hàng doanh nghiệp (trang Doanh nghiệp).
 *
 * Web tĩnh không có backend, nên form gửi tới dịch vụ form Formspree; Formspree chuyển tiếp
 * nội dung về email người sáng lập (email nhận được cài trong tài khoản Formspree, không nằm ở đây).
 */

/**
 * Địa chỉ nhận form, dạng `https://formspree.io/f/<mã form>`.
 *
 * TODO(owner): tạo form trên https://formspree.io bằng email người sáng lập rồi dán địa chỉ vào đây
 * (xem README). Khi còn để trống, trang Doanh nghiệp khóa form và hướng khách sang kênh chat.
 */
export const BUSINESS_CONTACT_FORM_ENDPOINT = '';

/** Địa chỉ nhận form nếu đã cấu hình hợp lệ (URL https), ngược lại `null`. */
export function configuredEndpoint(endpoint: string): string | null {
  try {
    return new URL(endpoint).protocol === 'https:' ? endpoint : null;
  } catch {
    return null;
  }
}

export interface SubmitResult {
  ok: boolean;
  /** Thông báo tiếng Việt hiển thị cho khách. */
  message: string;
}

const FALLBACK = 'Nếu vẫn không được, bạn nhắn cho mình qua các kênh chat ở Trang chủ nhé.';

/** Gửi nội dung form tới dịch vụ form và trả về kết quả kèm thông báo tiếng Việt cho khách. */
export async function submitContactForm(
  endpoint: string,
  data: FormData,
  fetchFn: typeof fetch = fetch,
): Promise<SubmitResult> {
  let response: Response;
  try {
    response = await fetchFn(endpoint, { method: 'POST', body: data, headers: { Accept: 'application/json' } });
  } catch {
    return { ok: false, message: `Không kết nối được, bạn kiểm tra mạng rồi thử lại. ${FALLBACK}` };
  }
  if (response.ok) {
    return { ok: true, message: 'Cảm ơn bạn! Mình đã nhận được thông tin và sẽ liên hệ lại sớm.' };
  }
  if (response.status >= 500) {
    return { ok: false, message: `Chưa gửi được vì dịch vụ form đang trục trặc, bạn thử lại sau ít phút. ${FALLBACK}` };
  }
  return { ok: false, message: `Chưa gửi được, bạn kiểm tra lại thông tin rồi thử lại. ${FALLBACK}` };
}
