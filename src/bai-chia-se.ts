import { getCollection, type CollectionEntry } from 'astro:content';

export type BaiChiaSe = CollectionEntry<'baiChiaSe'>;

/** Đường dẫn trang chi tiết của một Bài chia sẻ: tên file Markdown (không đuôi `.md`). */
export function baiChiaSeHref(bai: Pick<BaiChiaSe, 'id'>): string {
  return `/bai-chia-se/${bai.id}/`;
}

/** Mọi Bài chia sẻ trong `src/content/bai-chia-se/`. */
export async function getTatCaBaiChiaSe(): Promise<BaiChiaSe[]> {
  return getCollection('baiChiaSe');
}
