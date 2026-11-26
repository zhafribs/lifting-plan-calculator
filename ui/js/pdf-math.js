// The arithmetic behind the PDF page view: zoom, fitting and which pages keep a
// canvas. Kept free of the DOM so it can be tested without a webview.

export const ZOOM_MIN = 0.4;
export const ZOOM_MAX = 3;
export const ZOOM_STEP = 0.15;
/** A page is drawn at this multiple of its CSS size, capped so a 200% zoom on a
 *  hi-dpi screen does not allocate a canvas nobody can look at. */
export const MAX_BACKING = 2;

export function clampZoom(zoom) {
  if (!Number.isFinite(zoom)) return 1;
  return Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, zoom));
}

/** The zoom that makes a page as wide as the reader, minus a little gutter. */
export function fitWidth(containerWidth, pageWidth, padding = 48) {
  if (!(containerWidth > 0) || !(pageWidth > 0)) return 1;
  return clampZoom((containerWidth - padding) / pageWidth);
}

/** The page-first preview: an A4 box at the viewer's fit width, with the
 *  report's print margins, and the body scale the report is laid out at.
 *  PDF points and CSS pixels differ by 72/96, which is the easy mistake. */
export function pageFirstBox({
  viewerWidth,
  margins,
  pageWidth = 595.28,
  pageHeight = 841.89,
  padding = 48,
}) {
  if (!(viewerWidth > 0)) return null;
  const zoom = fitWidth(viewerWidth, pageWidth, padding);
  const ptPerMm = 72 / 25.4;
  const by = (mm) => mm * ptPerMm * zoom;
  return {
    zoom,
    width: pageWidth * zoom,
    height: pageHeight * zoom,
    padding: {
      top: by(margins.top),
      right: by(margins.right),
      bottom: by(margins.bottom),
      left: by(margins.left),
    },
    textZoom: zoom * (72 / 96),
  };
}

/** The zoom that puts a whole page in view. */
export function fitPage(containerWidth, containerHeight, page, padding = 48) {
  if (!(containerWidth > 0) || !(containerHeight > 0) || !page) return 1;
  const byWidth = (containerWidth - padding) / page.width;
  const byHeight = (containerHeight - padding) / page.height;
  return clampZoom(Math.min(byWidth, byHeight));
}

/** Canvas pixels per CSS pixel: the zoom, times the screen's density. */
export function backingScale(zoom, devicePixelRatio) {
  const density = Math.min(Math.max(devicePixelRatio || 1, 1), MAX_BACKING);
  return clampZoom(zoom) * density;
}

/** The zoom one step up or down, kept on a tidy grid. */
export function stepZoom(zoom, direction, step = ZOOM_STEP) {
  const next = clampZoom(zoom) + direction * step;
  return Math.round(clampZoom(next) * 100) / 100;
}

/**
 * The pages that may hold a canvas. Everything else is a sized placeholder, so a
 * long plan costs the pages on screen rather than the whole document.
 */
export function keptPages(current, count, radius = 2) {
  const keep = new Set();
  for (let index = current - radius; index <= current + radius; index += 1) {
    if (index >= 0 && index < count) keep.add(index);
  }
  return keep;
}

/** `page N of M`, with the page clamped into the document. */
export function pageLabel(page, count) {
  if (count <= 0) return "No pages";
  return `Page ${Math.min(Math.max(page, 1), count)} of ${count}`;
}
