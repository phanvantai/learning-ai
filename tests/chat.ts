const PENDING_LABEL = '(sắp có)';

/**
 * Các nút trong danh sách kênh Tư vấn qua chat của một trang: tên kênh, link (nếu bấm được) và có
 * đang ghi "sắp có" hay không. `undefined` nếu trang không có danh sách này.
 */
export function chatButtons(html: string) {
  const list = html.match(/<ul[^>]*aria-label="Kênh Tư vấn qua chat"[^>]*>([\s\S]*?)<\/ul>/)?.[1];
  if (list === undefined) return undefined;
  return [...list.matchAll(/<li[^>]*>([\s\S]*?)<\/li>/g)].map(([, item]) => {
    const text = item.replace(/<[^>]*>/g, ' ').replace(/\s+/g, ' ').trim();
    return {
      name: text.replace(PENDING_LABEL, '').trim(),
      href: item.match(/<a[^>]*href="([^"]*)"/)?.[1],
      pending: text.includes(PENDING_LABEL),
    };
  });
}
