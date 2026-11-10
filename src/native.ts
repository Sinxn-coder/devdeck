import type { Project } from './domain';
import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { open as openDirectory } from '@tauri-apps/plugin-dialog';

export const isNative = Boolean((window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__);

function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return tauriInvoke<T>(command, args);
}

export async function nativeProjects(): Promise<Project[]> {
  return invoke<Project[]>('list_projects');
}

export async function registerNativeProject(): Promise<Project | null> {
  const path = await openDirectory({ directory: true, multiple: false });
  return path && typeof path === 'string' ? invoke<Project>('register_project', { path }) : null;
}
export const toggleProjectPin = (id: string, pinned: boolean) => invoke<void>('toggle_project_pin', { id, pinned });

export type GitInfo = { is_repository: boolean; branch?: string; clean: boolean; staged: string[]; modified: string[]; untracked: string[]; recent_commits: string[] };
export type PortInfo = { port: number; protocol: string; process_id?: number; process_name?: string; process_attribution_supported: boolean };
export const inspectGit = (path: string) => invoke<GitInfo>('inspect_git', { path }).then((r)=>{recordGitHistory(path,r);return r});
export const discoverPorts = () => invoke<PortInfo[]>('discover_ports').then((r)=>{recordPortHistory(r);return r});
export type EnvReport = { files:string[]; expected:string[]; entries:{key:string;present:boolean;empty:boolean;line:number;error?:string}[]; missing:string[]; duplicate_keys:string[] };
export const inspectEnvironment = (path:string) => invoke<EnvReport>('inspect_environment',{path}).then((r)=>{recordEnvHistory(path,r);return r});

// history recording helpers call into the native-managed DB when available. Keep redaction local and avoid storing sensitive values.
import { recordHistory, safeSummary } from './history';
function recordGitHistory(path:string, info:GitInfo){ try{ if(isNative) return; const summary = info.is_repository ? `${info.branch||'detached'} · ${info.recent_commits.length} commits · ${info.staged.length} staged` : 'Not a git repository'; recordHistory({operation:'git.inspect',ok:true,summary:safeSummary(summary)}); }catch{} }
function recordPortHistory(ports:PortInfo[]){ try{ if(isNative) return; const summary = ports.length?`${ports.length} ports`:'No ports found'; recordHistory({operation:'ports.discover',ok:true,summary:safeSummary(summary)}); }catch{} }
function recordEnvHistory(path:string, report:EnvReport){ try{ if(isNative) return; const summary = `${report.files.length} files · ${report.missing.length} missing · ${report.duplicate_keys.length} duplicates`; recordHistory({operation:'env.inspect',ok:true,summary:safeSummary(summary)}); }catch{} }

export const readHistoryNative = () => invoke<any[]>('read_history');
export const clearHistoryNative = () => invoke<void>('clear_history');
export const recordHistoryNative = (operation: string, ok: boolean, summary: string, durationMs?: number) => invoke<void>('record_history', { operation, ok, summary, duration_ms: durationMs });
