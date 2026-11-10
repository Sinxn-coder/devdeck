declare module '@tauri-apps/api/core' { export function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>; }
declare module '@tauri-apps/plugin-dialog' { export function open(options?: {directory?: boolean; multiple?: boolean}): Promise<string | string[] | null>; }
