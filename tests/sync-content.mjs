import { execFileSync } from 'node:child_process';

/**
 * Vitest global setup: load the Markdown content collections before any test runs.
 *
 * Under Vitest, `astro:content` reads the data store from `.astro/` (like `astro dev`), but nothing
 * fills it there. Running `astro sync` with the cache pointed at `.astro/` writes it where the tests
 * look, so pages render the real Bài chia sẻ in `src/content/`. It runs in a child process because
 * `sync` switches `NODE_ENV` to production and Vite cannot load Astro's Node API inside Vitest.
 */
export default function syncContent() {
  execFileSync(
    process.execPath,
    [
      '--input-type=module',
      '--eval',
      "import { sync } from 'astro'; await sync({ cacheDir: './.astro/', logLevel: 'warn' });",
    ],
    { stdio: 'inherit' },
  );
}
