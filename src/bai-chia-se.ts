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

/** Mọi Bài chia sẻ trong `src/content/bai-chia-se/`, mới nhất trước. */
export async function getTatCaBaiChiaSe(): Promise<BaiChiaSe[]> {
  return moiNhatTruoc(await getCollection('baiChiaSe'));
}
