/** Một mục trên thanh điều hướng. */
export interface NavItem {
  label: string;
  href: string;
}

export const NAV_ITEMS: readonly NavItem[] = [
  { label: 'Trang chủ', href: '/' },
  { label: 'Bài chia sẻ', href: '/bai-chia-se/' },
  { label: 'Mentoring', href: '/mentoring/' },
  { label: 'Doanh nghiệp', href: '/doanh-nghiep/' },
  { label: 'Giới thiệu', href: '/gioi-thieu/' },
];

export const SITE = {
  name: 'Learning AI',
  description:
    'Kỹ sư phần mềm dùng AI mỗi ngày, chia sẻ những gì thực sự hiệu quả cho công việc của bạn.',
} as const;

/**
 * Thông tin người sáng lập. `TODO(owner)` đánh dấu nội dung thật người sáng lập cần tự điền.
 */
export const FOUNDER = {
  name: 'TODO(owner): tên thật của bạn',
} as const;

/** Một kênh nhắn tin cho Tư vấn qua chat. `href: null` nghĩa là chưa có link thật. */
export interface ChatChannel {
  name: string;
  href: string | null;
}

/**
 * Kênh Tư vấn qua chat, theo đúng thứ tự hiển thị trong `docs/specs/website.md`.
 * Đây là chỗ duy nhất cấu hình link chat. Kênh còn `null` vẫn hiện nhưng ghi "sắp có" và không bấm
 * được, để người xem trang không bao giờ gặp link hỏng.
 */
export const CHAT_CHANNELS: readonly ChatChannel[] = [
  // TODO(owner): link Zalo, dạng https://zalo.me/<số điện thoại>
  { name: 'Zalo', href: null },
  // TODO(owner): link Messenger, dạng https://m.me/<tên trang hoặc tài khoản>
  { name: 'Messenger', href: null },
  // TODO(owner): link Telegram, dạng https://t.me/<username>
  { name: 'Telegram', href: null },
  // TODO(owner): link LinkedIn, dạng https://www.linkedin.com/in/<tên hồ sơ>
  { name: 'LinkedIn', href: null },
];
