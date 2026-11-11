// @ts-check
import { defineConfig } from 'astro/config';

export default defineConfig({
  site: 'https://learning-ai-bay.vercel.app',
  markdown: {
    // Hai bộ màu cho khối code: sáng mặc định, tối qua biến `--shiki-dark` (xem `global.css`).
    shikiConfig: { themes: { light: 'github-light', dark: 'github-dark' } },
  },
});
