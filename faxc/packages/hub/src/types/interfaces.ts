export interface ComponentAdapter {
  execute(input: any): Promise<any>;
  validateInput(input: any): boolean;
  getName(): string;
  getVersion(): string;
}

export interface PipelineStage {
  name: string;
  adapter: ComponentAdapter;
  config: Record<string, any>;
}

export interface CompilationResult {
  success: boolean;
  data?: any;
  error?: string;
  time: number;
  warnings: string[];
}

export interface Config {
  sourcePath: string;
  targetLanguage: string;
  optLevel: number;
  debug: boolean;
  parallel: boolean;
  plugins: boolean;
  format: string;
  run: boolean;
}