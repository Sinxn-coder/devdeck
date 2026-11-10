export type HistoryEntry={id:string;at:string;operation:string;ok:boolean;summary:string;durationMs?:number};
const KEY='devdeck.history'; const LIMIT=100;
export function safeSummary(value:string):string{return value.replace(/(authorization|token|password|secret|api[_-]?key)\s*[:=]\s*[^\n]*/gi,'$1=[redacted]').slice(0,240)}
export function recordHistory(entry:Omit<HistoryEntry,'id'|'at'>):void{const current:HistoryEntry[]=JSON.parse(localStorage.getItem(KEY)||'[]');current.unshift({...entry,id:crypto.randomUUID(),at:new Date().toISOString(),summary:safeSummary(entry.summary)});localStorage.setItem(KEY,JSON.stringify(current.slice(0,LIMIT))); // mirror to native when available
try{if((window as any).__TAURI_INTERNALS__){window.dispatchEvent(new CustomEvent('devdeck-record-history',{detail:current[0]}))}}catch{} }
export function readHistory():HistoryEntry[]{try{return JSON.parse(localStorage.getItem(KEY)||'[]')}catch{return []}}
export function clearHistory():void{localStorage.removeItem(KEY);try{if((window as any).__TAURI_INTERNALS__){window.dispatchEvent(new CustomEvent('devdeck-clear-history'))}}catch{}}
