/** The destination shown before a transcript leaves the machine. */
export function providerHost(baseUrl: string): string {
  try {
    return new URL(baseUrl).host;
  } catch {
    return baseUrl;
  }
}

/** Invalid addresses are treated as remote so the privacy label fails safe. */
export function providerIsRemote(baseUrl: string): boolean {
  try {
    const host = new URL(baseUrl).hostname;
    return !['localhost', '127.0.0.1', '0.0.0.0', '[::1]'].includes(host);
  } catch {
    return true;
  }
}
