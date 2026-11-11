import { getCollection, type CollectionEntry } from 'astro:content';

export type BaiChiaSe = CollectionEntry<'baiChiaSe'>;

export type LoaiBai = BaiChiaSe['data']['loai'];

/** Hai Loại bài (CONTEXT.md), theo thứ tự hiển thị: bài cho Học viên cá nhân trước. */
export const LOAI_BAI: readonly { ma: LoaiBai; ten: string; moTa: string }[] = [
  { ma: 'ung-dung', ten: 'Ứng dụng', moTa: 'Dùng AI cho công việc hằng ngày. Không cần biết kỹ thuật.' },
  { ma: 'chuyen-sau', ten: 'Chuyên sâu', moTa: 'Sổ tay kỹ thuật về agent, harness, context. Viết cho người làm phần mềm.' },
];

/** Tên hiển thị của một Loại bài. */
export function tenLoai(ma: LoaiBai): string {
  return LOAI_BAI.find((loai) => loai.ma === ma)?.ten ?? ma;
}

/** Các bài thuộc một Loại bài, giữ nguyên thứ tự. */
export function baiTheoLoai<T extends { data: { loai: LoaiBai } }>(tatCaBai: readonly T[], ma: LoaiBai): T[] {
  return tatCaBai.filter((bai) => bai.data.loai === ma);
}

/** Đường dẫn trang danh sách mọi Bài chia sẻ. */
export const DANH_SACH_HREF = '/bai-chia-se/';

/** Đường dẫn trang chi tiết của một Bài chia sẻ: tên file Markdown (không đuôi `.md`). */
export function baiChiaSeHref(bai: Pick<BaiChiaSe, 'id'>): string {
  return `${DANH_SACH_HREF}${bai.id}/`;
}

/** Bản sao danh sách, bài mới nhất trước; bài cùng ngày xếp theo `id` để thứ tự luôn cố định. */
export function moiNhatTruoc<T extends { id: string; data: { date: Date } }>(tatCaBai: readonly T[]): T[] {
  return [...tatCaBai].sort((a, b) => b.data.date.getTime() - a.data.date.getTime() || a.id.localeCompare(b.id));
}

/** Mọi Bài chia sẻ trong `src/content/bai-chia-se/`, mới nhất trước. */
export async function getTatCaBaiChiaSe(): Promise<BaiChiaSe[]> {
  return moiNhatTruoc(await getCollection('baiChiaSe'));
}

/** Tốc độ đọc tiếng Việt trung bình, tính theo chữ (âm tiết) mỗi phút. */
const CHU_MOI_PHUT = 220;

/** Số phút đọc ước tính của nội dung Markdown, làm tròn lên, ít nhất 1 phút. */
export function soPhutDoc(noiDung: string): number {
  const soChu = noiDung.split(/\s+/).filter(Boolean).length;
  return Math.max(1, Math.ceil(soChu / CHU_MOI_PHUT));
}
