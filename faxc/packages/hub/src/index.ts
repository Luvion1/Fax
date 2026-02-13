import { PipelineBuilder } from './orchestrator/pipeline';
import { LexerAdapter } from './adapters/lexer_adapter';
import { ParserAdapter } from './adapters/parser_adapter';
import { SemanticAnalyzerAdapter } from './adapters/sema_adapter';
import { OptimizerAdapter } from './adapters/optimizer_adapter';
import { CodeGeneratorAdapter } from './adapters/codegen_adapter';
import { readFileSync } from 'fs';
import * as process from 'process';

async function main() {
  try {
    const args = process.argv.slice(2);
    if (args.length < 1) {
      console.error('Usage: npx ts-node src/index.ts <input_file.fax>');
      process.exit(1);
    }

    const inputFile = args[0];
    const sourceCode = readFileSync(inputFile, 'utf8');

    // Create pipeline with all adapters
    const pipeline = new PipelineBuilder()
      .addStage('lexer', new LexerAdapter())
      .addStage('parser', new ParserAdapter())
      .addStage('semantic-analyzer', new SemanticAnalyzerAdapter())
      .addStage('optimizer', new OptimizerAdapter())
      .addStage('code-generator', new CodeGeneratorAdapter())
      .build();

    // Execute the pipeline
    const result = await pipeline.execute(sourceCode);
    
    // Output the final result (LLVM IR)
    console.log(result);
  } catch (error) {
    console.error('Compilation failed:', error);
    process.exit(1);
  }
}

// Run the main function
main();