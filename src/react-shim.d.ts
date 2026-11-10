declare module 'react' {
  export type ReactNode = any;
  export type SetStateAction<T> = T | ((previous: T) => T);
  export type Dispatch<T> = (value: T) => void;
  export function useState<T>(initial: T | (() => T)): [T, (value: T | ((previous: T) => T)) => void];
  export function useEffect(effect: () => void | (() => void), dependencies?: unknown[]): void;
  export function useMemo<T>(factory: () => T, dependencies: unknown[]): T;
  const React: any; export default React;
}
declare module 'react-dom/client' { export function createRoot(element: Element): { render(node: any): void }; }
declare module 'react/jsx-runtime' { export const jsx: any; export const jsxs: any; export const Fragment: any; }
declare module '*.css' {}
declare namespace JSX { interface IntrinsicAttributes { key?: string | number } interface IntrinsicElements { [element: string]: any } }
