import { ComponentAdapter, CompilationResult } from '../types/interfaces';
import { execSync } from 'child_process';
import { writeFileSync } from 'fs';

export class CodeGeneratorAdapter implements ComponentAdapter {
  private readonly command: string;

  constructor(command: string = './faxc/packages/codegen/build/faxc_cpp') {
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
        tempFilePath = `.temp_codegen_input.json`;
        writeFileSync(tempFilePath, JSON.stringify(input));
      }

      const command = `${this.command} "${tempFilePath}"`;
      const result = execSync(command, { encoding: 'utf8', timeout: 300000 });
      
      // Return the raw result as codegen typically outputs LLVM IR
      return result.trim();
    } catch (error) {
      throw new Error(`Code generator execution failed: ${(error as Error).message}`);
    }
  }

  validateInput(input: any): boolean {
    return typeof input === 'string' || typeof input === 'object';
  }

  getName(): string {
    return 'code-generator';
  }

  getVersion(): string {
    return '1.0.0';
  }
}