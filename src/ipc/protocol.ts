/**
 * Builds `prose://` URLs for book resources (architecture section 4.3).
 *
 * This is the only sanctioned channel for book bytes: the renderer fetches
 * resources by URL so they never cross the `invoke` boundary. The scheme is
 * spelled differently per platform. macOS, iOS, and Linux use a real
 * `prose://` scheme; Windows and Android tunnel custom schemes through
 * `http://<scheme>.localhost/`.
 */

const useHttpScheme = /Windows|Android/i.test(navigator.userAgent)

/** Percent-encode a resource path while preserving its segment separators. */
function encodeResourcePath(path: string): string {
  return path
    .split('/')
    .map((segment) => encodeURIComponent(segment))
    .join('/')
}

/**
 * The `prose://book/{id}/{resourcePath}` URL for a book resource. With no
 * `resourcePath` it addresses the container itself, which the ePub renderer
 * unzips and the PDF renderer streams with range requests.
 */
export function bookResourceUrl(bookId: string, resourcePath = ''): string {
  const id = encodeURIComponent(bookId)
  const tail = resourcePath ? `/${encodeResourcePath(resourcePath)}` : ''
  return useHttpScheme
    ? `http://prose.localhost/book/${id}${tail}`
    : `prose://book/${id}${tail}`
}
