import { defineCollection } from 'astro:content';
import { glob } from 'astro/loaders';
import { z } from 'astro/zod';

const baiChiaSe = defineCollection({
  loader: glob({ pattern: '**/*.md', base: './src/content/bai-chia-se' }),
  schema: z.object({
    title: z.string(),
    date: z.coerce.date(),
    tags: z.array(z.string()),
    description: z.string(),
  }),
});

export const collections = { baiChiaSe };
