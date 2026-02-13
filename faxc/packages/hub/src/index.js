"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
const pipeline_1 = require("./orchestrator/pipeline");
const lexer_adapter_1 = require("./adapters/lexer_adapter");
const parser_adapter_1 = require("./adapters/parser_adapter");
const sema_adapter_1 = require("./adapters/sema_adapter");
const optimizer_adapter_1 = require("./adapters/optimizer_adapter");
const codegen_adapter_1 = require("./adapters/codegen_adapter");
const fs_1 = require("fs");
const process = __importStar(require("process"));
async function main() {
    try {
        const args = process.argv.slice(2);
        if (args.length < 1) {
            console.error('Usage: npx ts-node src/index.ts <input_file.fax>');
            process.exit(1);
        }
        const inputFile = args[0];
        const sourceCode = (0, fs_1.readFileSync)(inputFile, 'utf8');
        // Create pipeline with all adapters
        const pipeline = new pipeline_1.PipelineBuilder()
            .addStage('lexer', new lexer_adapter_1.LexerAdapter())
            .addStage('parser', new parser_adapter_1.ParserAdapter())
            .addStage('semantic-analyzer', new sema_adapter_1.SemanticAnalyzerAdapter())
            .addStage('optimizer', new optimizer_adapter_1.OptimizerAdapter())
            .addStage('code-generator', new codegen_adapter_1.CodeGeneratorAdapter())
            .build();
        // Execute the pipeline
        const result = await pipeline.execute(sourceCode);
        // Output the final result (LLVM IR)
        console.log(result);
    }
    catch (error) {
        console.error('Compilation failed:', error);
        process.exit(1);
    }
}
// Run the main function
main();
