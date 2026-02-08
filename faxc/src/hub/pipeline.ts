/**
 * FAX Compiler Hub - Main orchestrator for the polyglot compilation pipeline.
 * 
 * This module coordinates the execution of multiple compiler stages written in different languages:
 * - Lexer (Rust): Tokenization and UTF-8 handling
 * - Parser (Zig): Syntax analysis and AST generation
 * - Semantic Analyzer (Haskell): Type checking and scope validation
 * - Optimizer (Python): AST transformations and metadata annotation
 * - Code Generator (C++): LLVM IR generation
 * - Native Compiler (Zig CC): Final binary compilation
 */

import { execSync } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import { Token, ASTNode, CompilerConfig } from '../common/types';

/**
 * FaxCompilerHub orchestrates the entire compilation pipeline.
 * Manages communication between polyglot compiler components via JSON interchange format.
 */
class FaxCompilerHub {
  private tokens: Token[] = [];
  private ast: ASTNode | null = null;
  private typedAst: ASTNode | null = null;
  private config: CompilerConfig;

  constructor(config: CompilerConfig) {
    this.config = config;
  }

  /**
   * Print a formatted compilation step message with color coding.
   * @param action - The action being performed (e.g., "Compiling", "Building")
   * @param message - The message content
   * @param color - ANSI color code (default: green)
   */
  private printStep(action: string, message: string, color: string = '\x1b[1;32m'): void {
    const paddedAction = action.padStart(12);
    console.log(`${color}${paddedAction}\x1b[0m ${message}`);
  }

  /**
   * Recursively search for Fax.toml configuration file starting from given directory.
   * @param startDir - Directory to begin search
   * @returns Path to Fax.toml if found, null otherwise
   */
  private findFaxToml(startDir: string): string | null {
    let current = startDir;
    while (true) {
      const configPath = path.join(current, 'Fax.toml');
      if (fs.existsSync(configPath)) return configPath;
      
      const parent = path.dirname(current);
      if (parent === current) break; // Reached filesystem root
      current = parent;
    }
    return null;
  }

  /**
   * Parse Fax.toml configuration file into structured object.
   * Supports [package] and [profile.*] sections with key=value pairs.
   * @param content - Raw TOML content as string
   * @returns Parsed configuration object
   */
  private parseFaxToml(content: string): Record<string, any> {
    const result: Record<string, any> = { 
      package: {}, 
      profile: { dev: {}, release: {} } 
    };
    
    let currentSection = '';
    const lines = content.split('\n');
    
    for (const rawLine of lines) {
      const line = rawLine.trim();
      if (!line || line.startsWith('#')) continue; // Skip empty lines and comments
      
      // Parse section headers [section.name]
      if (line.startsWith('[') && line.endsWith(']')) {
        currentSection = line.slice(1, -1);
        continue;
      }
      
      // Parse key=value pairs
      const parts = line.split('=');
      if (parts.length < 2) continue;
      
      const key = parts[0].trim();
      const value = parts.slice(1).join('=').trim().replace(/^"(.*)"$/, '$1');
      
      if (currentSection === 'package') {
        result.package[key] = value;
      } else if (currentSection.startsWith('profile.')) {
        const profileName = currentSection.split('.')[1];
        if (!result.profile[profileName]) result.profile[profileName] = {};
        result.profile[profileName][key] = value;
      }
    }
    return result;
  }

  /**
   * Main compilation entry point. Orchestrates the entire pipeline:
   * 1. Configuration loading (Fax.toml)
   * 2. Lexical analysis (Rust)
   * 3. Parsing (Zig)
   * 4. Semantic analysis (Haskell)
   * 5. Optimization (Python)
   * 6. Code generation (C++)
   * 7. Native compilation (Zig CC)
   * 
   * @returns Path to generated binary on success
   * @throws Error on compilation failure at any stage
   */
  public async start(): Promise<string> {
    const startTime = Date.now();
    try {
      const isRelease = process.argv.includes('--release');
      const profileName = isRelease ? 'release' : 'dev';
      
      // Set default project metadata
      let projectName = 'fax';
      let version = '0.0.1';
      let projectRoot = process.cwd();
      let profileConfig = isRelease 
        ? { 'opt-level': '3', 'debug': 'false' } 
        : { 'opt-level': '0', 'debug': 'true' };
      
      // Locate source file from command line arguments
      const sourcePathArg = process.argv.find(arg => arg.endsWith('.fax')) || 'example.fax';
      this.config.sourcePath = sourcePathArg;

      // Search for Fax.toml configuration
      const sourceDir = path.dirname(path.resolve(this.config.sourcePath));
      const tomlPath = this.findFaxToml(sourceDir);
      
      if (tomlPath) {
        projectRoot = path.dirname(tomlPath);
        const toml = this.parseFaxToml(fs.readFileSync(tomlPath, 'utf8'));
        projectName = toml.package.name || projectName;
        version = toml.package.version || version;
        if (toml.profile && toml.profile[profileName]) {
          profileConfig = { ...profileConfig, ...toml.profile[profileName] };
        }
      }

      // Fetch optional Git information
      let gitInfo = '';
      try {
        const branch = execSync('git rev-parse --abbrev-ref HEAD', { 
          stdio: 'pipe', 
          cwd: projectRoot 
        }).toString().trim();
        const hash = execSync('git rev-parse --short HEAD', { 
          stdio: 'pipe', 
          cwd: projectRoot 
        }).toString().trim();
        gitInfo = ` (\x1b[33m${branch}\x1b[0m \x1b[90m${hash}\x1b[0m)`;
      } catch (e) {
        // Git not available or not a repository - continue without git info
      }

      // Print compilation header
      const sourceFile = path.relative(projectRoot, this.config.sourcePath);
      this.printStep(
        'Compiling', 
        `${projectName} v${version} (${projectRoot})${gitInfo}`
      );
      
      // Display optimization and debug settings
      const optStatus = profileConfig['opt-level'] === '0' 
        ? 'unoptimized' 
        : `optimized [O${profileConfig['opt-level']}]`;
      const debugStatus = profileConfig['debug'] === 'true' 
        ? 'debuginfo' 
        : 'no debuginfo';
      
      this.printStep(
        'Building',
        `${sourceFile} [${profileName}: ${optStatus} + ${debugStatus}]`,
        '\x1b[1;34m'
      );

      // STAGE 1: Lexical Analysis (Rust)
      this.tokens = await this.callComponent('lexer', this.config.sourcePath);

      // STAGE 2: Parsing (Zig)
      this.ast = await this.callComponent('parser', this.tokens);

      // STAGE 3: Semantic Analysis (Haskell)
      this.typedAst = await this.callComponent('sema', this.ast);
      if (this.typedAst && (this.typedAst as any).error) {
        throw new Error((this.typedAst as any).error);
      }

      // STAGE 4: Optimization (Python)
      this.typedAst = await this.callComponent('optimizer', this.typedAst);

      // STAGE 5: Code Generation (C++)
      const llvmIR = await this.callComponent('codegen', this.typedAst);
      
      // Write LLVM IR to file
      const baseName = path.basename(this.config.sourcePath, '.fax');
      const llPath = `${baseName}.ll`;
      const binPath = `./${baseName}`;

      fs.writeFileSync(llPath, llvmIR);

      // STAGE 6: Native Compilation (Zig CC)
      this.compileNative(llPath, binPath, profileConfig);

      // Report successful compilation
      const duration = ((Date.now() - startTime) / 1000).toFixed(2);
      this.printStep('Finished', `${profileName} target(s) in ${duration}s`);
      
      return binPath;
    } catch (error: any) {
      console.error(`\n\x1b[1;31merror\x1b[0m: \x1b[1mfailed to compile \x1b[0m${this.config.sourcePath}`);
      process.exit(1);
    }
  }

  /**
   * Compile LLVM IR to native binary using Zig compiler toolchain.
   * Includes linking with Fgc runtime for garbage collection.
   * @param llPath - Path to LLVM IR file
   * @param binPath - Output binary path
   * @param profileConfig - Compiler profile configuration
   * @throws Error on compilation failure
   */
  private compileNative(
    llPath: string, 
    binPath: string, 
    profileConfig: Record<string, any>
  ): void {
    const gcObj = 'fgc.o';
    let zigBuildOpt = 'Debug';
    const optLevel = profileConfig['opt-level'];
    
    // Map optimization levels to Zig build modes
    if (optLevel === '3') zigBuildOpt = 'ReleaseFast';
    else if (optLevel === '2') zigBuildOpt = 'ReleaseSafe';
    else if (optLevel === 's') zigBuildOpt = 'ReleaseSmall';
    
    const optFlag = `-O${optLevel}`;
    const debugFlag = profileConfig['debug'] === 'true' ? '-g' : '';
    
    try {
      // Compile Fgc (Fax Garbage Collector) runtime object file
      execSync(
        `zig build-obj faxc/src/runtime/fgc.zig -femit-bin=${gcObj} -fPIE -lc -O ${zigBuildOpt}`
      );
      
      // Link LLVM IR with Fgc runtime to produce final binary
      execSync(
        `zig cc ${llPath} ${gcObj} -o ${binPath} -Wno-override-module -pie -lc ${optFlag} ${debugFlag}`
      );
      
       // Clean up temporary object file
       if (fs.existsSync(gcObj)) fs.unlinkSync(gcObj);
     } catch (error: any) {
       throw new Error(`Native compilation failed: ${error.message}`);
     }
  }

  /**
   * Invoke a compiler component (lexer, parser, sema, optimizer, codegen).
   * Components are separate executables that communicate via JSON.
   * @param name - Component name
   * @param input - Component input (file path or structured data)
   * @returns Parsed component output
   */
  private async callComponent(name: string, input: any): Promise<any> {
    const uniqueId = Math.random().toString(36).substring(7);
    const tempInputFile = path.join(process.cwd(), `.temp_${name}_${uniqueId}_input.json`);
    const isFilePathInput = typeof input === 'string' && fs.existsSync(input);

    // Write input to temporary file if it's not already a file path
    if (!isFilePathInput) {
      fs.writeFileSync(tempInputFile, JSON.stringify(input, null, 2));
    }

    const inputArg = isFilePathInput ? input : tempInputFile;
    
    // Build component invocation command
    let command = '';
    switch (name) {
      case 'lexer':
        command = `cargo run --quiet --bin lexer -- "${inputArg}"`;
        break;
      case 'parser':
        command = `zig run faxc/src/components/parser/parser.zig -- "${inputArg}"`;
        break;
      case 'sema':
        command = `ghc -dynamic faxc/src/components/sema/sema.hs -o faxc/src/components/sema/sema_bin && ./faxc/src/components/sema/sema_bin "${inputArg}"`;
        break;
      case 'optimizer':
        command = `python3 faxc/src/components/optimizer/optimizer.py "${inputArg}"`;
        break;
      case 'codegen':
        command = `g++ faxc/src/components/codegen/codegen.cpp -o faxc/src/components/codegen/codegen_bin && ./faxc/src/components/codegen/codegen_bin "${inputArg}"`;
        break;
      default:
        throw new Error(`Unknown component: ${name}`);
    }

    try {
      const stdout = execSync(command, { stdio: ['inherit', 'pipe', 'pipe'] });
      return this.parseComponentOutput(name, stdout.toString().trim());
    } catch (error: any) {
      // Forward stderr to user
      if (error.stderr && error.stderr.toString().trim()) {
        process.stderr.write(error.stderr.toString());
      }
      // Try to parse stdout even on failure
      if (error.stdout) {
        const result = this.parseComponentOutput(name, error.stdout.toString().trim());
        if (result && result.error) return result;
      }
      process.exit(1);
    } finally {
      // Clean up temporary input file
      if (fs.existsSync(tempInputFile)) fs.unlinkSync(tempInputFile);
    }
  }

  /**
   * Parse component output and extract JSON data.
   * Handles cases where component output includes debug messages before/after JSON.
   * @param name - Component name
   * @param outputStr - Raw output string from component
   * @returns Parsed JSON object or raw string for codegen
   */
  private parseComponentOutput(name: string, outputStr: string): any {
    // Codegen returns raw LLVM IR text
    if (name === 'codegen') return outputStr;
    
    // Find JSON structure in output
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
    } catch (error) {
      return outputStr; // Return raw output if JSON parsing fails
    }
  }
}

const config: CompilerConfig = {
  sourcePath: process.argv[2] || 'example.fax',
  targetLanguage: 'rs'
};

// Create example file if not exists
if (!fs.existsSync(config.sourcePath)) {
  fs.writeFileSync(config.sourcePath, 'fn main() { print("Hello Fax"); }');
}

// Start compilation
new FaxCompilerHub(config).start();