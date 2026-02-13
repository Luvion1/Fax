"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.CodeGeneratorAdapter = void 0;
const child_process_1 = require("child_process");
const fs_1 = require("fs");
class CodeGeneratorAdapter {
    constructor(command = './faxc/packages/codegen/build/faxc_cpp') {
        this.command = command;
    }
    async execute(input) {
        let tempFilePath;
        try {
            if (typeof input === 'string') {
                // Input is a file path
                tempFilePath = input;
            }
            else {
                // Input is an object, write to temporary file
                tempFilePath = `.temp_codegen_input.json`;
                (0, fs_1.writeFileSync)(tempFilePath, JSON.stringify(input));
            }
            const command = `${this.command} "${tempFilePath}"`;
            const result = (0, child_process_1.execSync)(command, { encoding: 'utf8', timeout: 300000 });
            // Return the raw result as codegen typically outputs LLVM IR
            return result.trim();
        }
        catch (error) {
            throw new Error(`Code generator execution failed: ${error.message}`);
        }
    }
    validateInput(input) {
        return typeof input === 'string' || typeof input === 'object';
    }
    getName() {
        return 'code-generator';
    }
    getVersion() {
        return '1.0.0';
    }
}
exports.CodeGeneratorAdapter = CodeGeneratorAdapter;
