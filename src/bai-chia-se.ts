import { getCollection, type CollectionEntry } from 'astro:content';

export type BaiChiaSe = CollectionEntry<'baiChiaSe'>;

/** Đường dẫn trang chi tiết của một Bài chia sẻ: tên file Markdown (không đuôi `.md`). */
export function baiChiaSeHref(bai: Pick<BaiChiaSe, 'id'>): string {
  return `/bai-chia-se/${bai.id}/`;
}

/** Bản sao danh sách, bài mới nhất trước; bài cùng ngày xếp theo `id` để thứ tự luôn cố định. */
export function moiNhatTruoc<T extends { id: string; data: { date: Date } }>(tatCaBai: readonly T[]): T[] {
  return [...tatCaBai].sort((a, b) => b.data.date.getTime() - a.data.date.getTime() || a.id.localeCompare(b.id));
}

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
  for (const ten of tatCaBai.flatMap((bai) => bai.data.tags)) {
    const slug = chuDeSlug(ten);
    if (!theoSlug.has(slug)) theoSlug.set(slug, { ten, slug });
  }
  return [...theoSlug.values()].sort((a, b) => a.ten.localeCompare(b.ten, 'vi'));
}

/** Các bài có thẻ chủ đề mang `slug`, giữ nguyên thứ tự. */
export function baiThuocChuDe<T extends { data: { tags: readonly string[] } }>(tatCaBai: readonly T[], slug: string): T[] {
  return tatCaBai.filter((bai) => bai.data.tags.some((ten) => chuDeSlug(ten) === slug));
}

/** Mọi Bài chia sẻ trong `src/content/bai-chia-se/`, mới nhất trước. */
export async function getTatCaBaiChiaSe(): Promise<BaiChiaSe[]> {
  return moiNhatTruoc(await getCollection('baiChiaSe'));
}
