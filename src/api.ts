export type Pair = { key: string; value: string };
export function buildUrl(raw: string, query: Pair[]): string {
  const url = new URL(raw);
  query.filter(p => p.key.trim()).forEach(p => url.searchParams.set(p.key.trim(), p.value));
  return url.toString();
}
export function formatResponse(body: string, contentType = ''): string {
  if (contentType.includes('application/json')) {
    try { return JSON.stringify(JSON.parse(body), null, 2); } catch { return body; }
  }
  return body;
}
