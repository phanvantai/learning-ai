import { experimental_AstroContainer as AstroContainer } from 'astro/container';
import type { AstroComponentFactory } from 'astro/runtime/server/index.js';

/**
 * Render a page component to its HTML string, the way a visitor requesting `path` would receive it.
 * Dynamic routes (e.g. `[id].astro`) also need the route `params` matched from that path.
 */
export async function renderPage(
  page: AstroComponentFactory,
  path = '/',
  params?: Record<string, string>,
): Promise<string> {
  const container = await AstroContainer.create();
  return container.renderToString(page, { request: new Request(new URL(path, 'https://example.test')), params });
}

/** The visible main content of a rendered page (what is inside `<main>`). */
export function mainContent(html: string): string {
  const match = html.match(/<main[^>]*>([\s\S]*)<\/main>/);
  if (!match) throw new Error('Page has no <main> element');
  return match[1];
}

/**
 * All `<a>` links inside the `<nav>` with `aria-label`, as `{ label, href, current }`.
 * `current` is true for the page being viewed (`aria-current="page"`) or its section (`"true"`).
 */
export function navLinks(html: string, ariaLabel: string) {
  const nav = html.match(new RegExp(`<nav[^>]*aria-label="${ariaLabel}"[^>]*>([\\s\\S]*?)</nav>`))?.[1] ?? '';
  return [...nav.matchAll(/<a([^>]*)>\s*([^<]*?)\s*<\/a>/g)].map(([, attrs, label]) => ({
    label,
    href: attrs.match(/href="([^"]*)"/)?.[1],
    current: /aria-current="(page|true)"/.test(attrs),
  }));
}

/** Render a single component with `props`, for states a page cannot be put into from a test. */
export async function renderComponent(component: AstroComponentFactory, props: Record<string, unknown>): Promise<string> {
  const container = await AstroContainer.create();
  return container.renderToString(component, { props });
}
