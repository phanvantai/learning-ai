/**
 * Thẻ chủ đề của Bài chia sẻ. Module thuần (không phụ thuộc `astro:content`) để cả schema trong
 * `src/content.config.ts` cũng dùng được.
 */

/** Một thẻ chủ đề: tên hiển thị như người viết ghi trong frontmatter, và slug dùng trên đường dẫn. */
export interface ChuDe {
  ten: string;
  slug: string;
}

/** Slug của thẻ chủ đề: bỏ dấu tiếng Việt, chữ thường, nối bằng gạch ngang ("Prompt mẫu" → "prompt-mau"). */
export function chuDeSlug(ten: string): string {
  return ten
    .normalize('NFD')
    .replace(/\p{M}/gu, '')
    .replace(/đ/gi, 'd')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '');
}

/** Thẻ chủ đề ứng với tên người viết ghi trong frontmatter. */
export function taoChuDe(ten: string): ChuDe {
  return { ten, slug: chuDeSlug(ten) };
}

/** Đường dẫn trang liệt kê các Bài chia sẻ có một thẻ chủ đề. */
export function chuDeHref(chuDe: Pick<ChuDe, 'slug'>): string {
  return `/bai-chia-se/chu-de/${chuDe.slug}/`;
}

/**
 * Mọi thẻ chủ đề đang dùng, xếp theo ABC. Thẻ cùng slug (khác hoa thường hay dấu) được gộp làm một,
 * giữ cách viết gặp đầu tiên.
 */
export function tatCaChuDe(tatCaBai: readonly { data: { tags: readonly string[] } }[]): ChuDe[] {
  const theoSlug = new Map<string, ChuDe>();
  for (const chuDe of tatCaBai.flatMap((bai) => bai.data.tags.map(taoChuDe))) {
    if (!theoSlug.has(chuDe.slug)) theoSlug.set(chuDe.slug, chuDe);
  }
  return [...theoSlug.values()].sort((a, b) => a.ten.localeCompare(b.ten, 'vi'));
}

/** Các bài có thẻ chủ đề mang `slug`, giữ nguyên thứ tự. */
export function baiThuocChuDe<T extends { data: { tags: readonly string[] } }>(tatCaBai: readonly T[], slug: string): T[] {
  return tatCaBai.filter((bai) => bai.data.tags.some((ten) => chuDeSlug(ten) === slug));
}
