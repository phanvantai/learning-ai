import { chuDeSlug } from '../src/chu-de';

/** Đọc những gì người xem thấy về Bài chia sẻ trong HTML đã render. */

/** Mỗi Bài chia sẻ được liệt kê, theo thứ tự trên trang: `id`, ngày đăng (`YYYY-MM-DD`) và HTML của thẻ bài. */
export function listedBai(html: string): { id: string; date: string; html: string }[] {
  return html
    .split('<article')
    .slice(1)
    .map((card) => ({
      id: card.match(/href="\/bai-chia-se\/([^/"]+)\/"/)?.[1] ?? '',
      date: card.match(/<time datetime="([^"]+)"/)?.[1] ?? '',
      html: card,
    }));
}

/** Ngày đăng có xếp từ mới đến cũ không. */
export function isNewestFirst(dates: string[]): boolean {
  return dates.every((date, i) => i === 0 || dates[i - 1] >= date);
}

/** Slug của mọi thẻ chủ đề đang dùng trong các bài. */
export function slugsInUse(tatCaBai: readonly { data: { tags: string[] } }[]): Set<string> {
  return new Set(tatCaBai.flatMap((bai) => bai.data.tags.map(chuDeSlug)));
}
