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
