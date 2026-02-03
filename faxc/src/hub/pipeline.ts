import { execSync } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import { Token, ASTNode, CompilerConfig } from '../common/types';

class FaxCompilerHub {
  private tokens: Token[] = [];
  private ast: ASTNode | null = null;
  private typedAst: ASTNode | null = null;

  constructor(private config: CompilerConfig) {}

  private printStep(action: string, message: string, color: string = '\x1b[1;32m') {
    const paddedAction = action.padStart(12);
    console.log(`${color}${paddedAction}\x1b[0m ${message}`);
  }

  public async start() {
    const startTime = Date.now();
    try {
      const fileName = path.basename(this.config.sourcePath);
      this.printStep('Compiling', `fax-project v1.0.0 (/root/Fax-lang)`);

      // 1. LEXING phase
      this.tokens = await this.callComponent('lexer', this.config.sourcePath);

      // 2. PARSING phase
      this.ast = await this.callComponent('parser', this.tokens);

      // 3. SEMANTIC phase
      this.typedAst = await this.callComponent('sema', this.ast);
      if (this.typedAst && this.typedAst.error) {
          throw new Error(this.typedAst.error);
      }

      // 3.5 OPTIMIZATION phase (Python)
      this.typedAst = await this.callComponent('optimizer', this.typedAst);

      // 4. CODEGEN phase (C++ generating LLVM IR)
      const llvmIR = await this.callComponent('codegen', this.typedAst);
      
      const baseName = path.basename(this.config.sourcePath, '.fax');
      const llPath = `${baseName}.ll`;
      const binPath = `./${baseName}`;

      fs.writeFileSync(llPath, llvmIR);

      // 5. NATIVE COMPILATION phase (Link using zig cc)
      try {
          const gcObj = "fgc.o";
          execSync(`zig build-obj src/runtime/fgc.zig -femit-bin=${gcObj} -fPIE -lc`);
          execSync(`zig cc ${llPath} ${gcObj} -o ${binPath} -Wno-override-module -pie -lc`);
          if (fs.existsSync(gcObj)) fs.unlinkSync(gcObj);
      } catch (e: any) {
          throw e;
      }

      // 6. CLEANUP
      if (fs.existsSync(llPath)) {
          // fs.unlinkSync(llPath);
      }

      const duration = ((Date.now() - startTime) / 1000).toFixed(2);
      this.printStep('Finished', `dev [unoptimized + debuginfo] target(s) in ${duration}s`);
      
      return binPath;
    } catch (error: any) {
      console.error(`\n\x1b[1;31merror\x1b[0m: \x1b[1mfailed to compile \x1b[0m${this.config.sourcePath}`);
      // Error detail from components usually handled by stderr piping in callComponent
      process.exit(1);
    }
  }

  private async callComponent(name: string, input: any): Promise<any> {
    const uniqueId = Math.random().toString(36).substring(7);
    const tempInputFile = path.join(process.cwd(), `.temp_${name}_${uniqueId}_input.json`);
    const isFilePathInput = typeof input === 'string' && fs.existsSync(input);

    if (!isFilePathInput) {
      fs.writeFileSync(tempInputFile, JSON.stringify(input, null, 2));
    }

    const inputArg = isFilePathInput ? input : tempInputFile;
    
    let command = '';
    switch (name) {
      case 'lexer': command = `cargo run --quiet --bin lexer -- "${inputArg}"`; break;
      case 'parser': command = `zig run src/components/parser/parser.zig -- "${inputArg}"`; break;
      case 'sema': command = `ghc -dynamic src/components/sema/sema.hs -o src/components/sema/sema_bin && ./src/components/sema/sema_bin "${inputArg}"`; break;
      case 'optimizer': command = `python3 src/components/optimizer/optimizer.py "${inputArg}"`; break;
      case 'codegen': command = `g++ src/components/codegen/codegen.cpp -o src/components/codegen/codegen_bin && ./src/components/codegen/codegen_bin "${inputArg}"`; break;
      default: throw new Error(`Unknown component: ${name}`);
    }

    try {
      const stdout = execSync(command, { stdio: ['inherit', 'pipe', 'pipe'] });
      return this.parseComponentOutput(name, stdout.toString().trim());
    } catch (e: any) {
      if (e.stderr && e.stderr.toString().trim()) {
          process.stderr.write(e.stderr.toString());
      }
      if (e.stdout) {
          const res = this.parseComponentOutput(name, e.stdout.toString().trim());
          if (res && res.error) return res;
      }
      process.exit(1);
    } finally {
      if (fs.existsSync(tempInputFile)) fs.unlinkSync(tempInputFile);
    }
  }

  private parseComponentOutput(name: string, outputStr: string): any {
      if (name === 'codegen') return outputStr;
      
      const firstCurly = outputStr.indexOf('{');
      const firstSquare = outputStr.indexOf('[');
      let startIdx = -1;
      if (firstCurly === -1) startIdx = firstSquare;
      else if (firstSquare === -1) startIdx = firstCurly;
      else startIdx = Math.min(firstCurly, firstSquare);

      const lastCurly = outputStr.lastIndexOf('}');
      const lastSquare = outputStr.lastIndexOf(']');
      const endIdx = Math.max(lastCurly, lastSquare);

      if (startIdx !== -1 && endIdx !== -1 && endIdx > startIdx) {
          outputStr = outputStr.substring(startIdx, endIdx + 1);
      }
      
      try {
        return JSON.parse(outputStr);
      } catch (e) {
        return outputStr;
      }
  }
}

const config: CompilerConfig = {
  sourcePath: process.argv[2] || 'example.fax',
  targetLanguage: 'rs'
};

if (!fs.existsSync(config.sourcePath)) {
  fs.writeFileSync(config.sourcePath, 'fn main() { print("Hello Fax"); }');
}

new FaxCompilerHub(config).start();