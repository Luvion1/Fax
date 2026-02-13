import { ComponentAdapter, CompilationResult } from '../types/interfaces';
import { execSync } from 'child_process';
import { writeFileSync } from 'fs';

export class ParserAdapter implements ComponentAdapter {
  private readonly command: string;

  constructor(command: string = '../parser/zig-out/bin/parser') {
    this.command = command;
  }

  async execute(input: any): Promise<any> {
    let tempFilePath: string | undefined;
    
    try {
      if (typeof input === 'string') {
        // Input is a file path
        tempFilePath = input;
      } else {
        // Input is an object, write to temporary file
        tempFilePath = `.temp_parser_input.json`;
        writeFileSync(tempFilePath, JSON.stringify(input));
      }

      const command = `${this.command} "${tempFilePath}"`;
      const result = execSync(command, { encoding: 'utf8', timeout: 300000 });
      
      // Try to parse the result as JSON
      try {
        return JSON.parse(result.trim());
      } catch {
        // If parsing fails, return the raw result
        return result.trim();
      }
    } catch (error) {
      throw new Error(`Parser execution failed: ${(error as Error).message}`);
    }
  }

  validateInput(input: any): boolean {
    return typeof input === 'string' || typeof input === 'object';
  }

  getName(): string {
    return 'parser';
  }

  getVersion(): string {
    return '1.0.0';
  }
}