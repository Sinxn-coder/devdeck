import { describe, expect, it } from 'vitest';
import { buildUrl, formatResponse } from './api';
describe('RequestBox helpers', () => {
  it('encodes query parameters', () => expect(buildUrl('https://example.test/a', [{key:'q',value:'a b'}, {key:'empty',value:''}])).toBe('https://example.test/a?q=a+b&empty='));
  it('formats JSON without executing it', () => expect(formatResponse('{"ok":true}', 'application/json')).toContain('"ok": true'));
  it('preserves non-json response text', () => expect(formatResponse('<script>alert(1)</script>', 'text/html')).toContain('<script>'));
});
