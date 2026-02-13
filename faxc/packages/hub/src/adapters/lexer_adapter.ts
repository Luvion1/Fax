import { ComponentAdapter, CompilationResult } from '../types/interfaces';
import { execSync } from 'child_process';
import { writeFileSync } from 'fs';

export class LexerAdapter implements ComponentAdapter {
  private readonly command: string;

  constructor(command: string = '../lexer/target/release/lexer') {
    this.command = command;
  }

  async execute(input: any): Promise<any> {
    const tempFilePath = `.temp_lexer_input.fax`;
    
    try {
      if (typeof input === 'string') {
        writeFileSync(tempFilePath, input);
      } else {
        writeFileSync(tempFilePath, JSON.stringify(input));
      }

      const command = `${this.command} "${tempFilePath}"`;
      const result = execSync(command, { encoding: 'utf8', timeout: 300000 });
      
      // Cleanup temp file
      try { require('fs').unlinkSync(tempFilePath); } catch {}
      
      // Try to parse the result as JSON
      try {
        return JSON.parse(result.trim());
      } catch {
        return result.trim();
      }
    } catch (error) {
      try { require('fs').unlinkSync(tempFilePath); } catch {}
      throw new Error(`Lexer execution failed: ${(error as Error).message}`);
    }
  }

  validateInput(input: any): boolean {
    return typeof input === 'string' || typeof input === 'object';
  }

  getName(): string {
    return 'lexer';
  }

  getVersion(): string {
    return '1.0.0';
  }
}