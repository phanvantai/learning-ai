import { defineCollection } from 'astro:content';
import { glob } from 'astro/loaders';
import { z } from 'astro/zod';
import { chuDeSlug } from './chu-de';

const tenChuDe = z
  .string()
  .refine((ten) => chuDeSlug(ten) !== '', 'Thẻ chủ đề cần có ít nhất một chữ cái hoặc chữ số');

/** Bài chia sẻ: mỗi file Markdown trong `src/content/bai-chia-se/` là một bài. Cách viết: xem README. */
const baiChiaSe = defineCollection({
  loader: glob({ pattern: '**/*.md', base: './src/content/bai-chia-se' }),
  schema: z.object({
    title: z.string().min(1),
    date: z.coerce.date(),
    tags: z.array(tenChuDe).min(1),
    description: z.string().min(1),
  }),
});

export const collections = { baiChiaSe };
