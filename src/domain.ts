export type Project={id:string; name:string; path:string; tech:string; git:boolean; pinned:boolean; updatedAt:string};
export function detectProject(files:string[], packageJson?:{dependencies?:Record<string,unknown>;devDependencies?:Record<string,unknown>;scripts?:Record<string,string>}):string[]{
 const set=new Set<string>(); const all={...(packageJson?.dependencies||{}),...(packageJson?.devDependencies||{})};
 if(files.includes('pubspec.yaml')) set.add('Flutter'); if(files.some(f=>f.endsWith('.py'))||files.includes('pyproject.toml')||files.includes('requirements.txt')) set.add('Python');
 if(files.includes('next.config.js')||files.includes('next.config.mjs')||all.next) set.add('Next.js');
 if(files.includes('vite.config.ts')||files.includes('vite.config.js')||all.vite) set.add('Vite');
 if(all.react) set.add('React'); if(files.includes('tsconfig.json')||all.typescript) set.add('TypeScript'); if(packageJson||files.includes('package.json')) set.add('Node.js'); if(files.includes('.git')) set.add('Git');
 return [...set].length?[...set]:['Generic project'];
}
