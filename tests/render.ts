import { experimental_AstroContainer as AstroContainer } from 'astro/container';
import type { AstroComponentFactory } from 'astro/runtime/server/index.js';

/**
 * Render a page component to its HTML string, the way a visitor requesting `path` would receive it.
 */
export async function renderPage(page: AstroComponentFactory, path = '/'): Promise<string> {
  const container = await AstroContainer.create();
  return container.renderToString(page, { request: new Request(new URL(path, 'https://example.test')) });
}

/** The visible main content of a rendered page (what is inside `<main>`). */
export function mainContent(html: string): string {
  const match = html.match(/<main[^>]*>([\s\S]*)<\/main>/);
  if (!match) throw new Error('Page has no <main> element');
  return match[1];
}
