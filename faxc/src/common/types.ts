export interface Token {
  type: string;
  value: string;
  line: number;
  column: number;
}

export interface ASTNode {
  type: string;
  [key: string]: any;
}

export interface CompilerConfig {
  sourcePath: string;
  outputPath?: string;
  targetLanguage: 'rs' | 'zig' | 'cpp' | 'py' | 'hs';
}
