"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.SemanticAnalyzerAdapter = void 0;
const child_process_1 = require("child_process");
const fs_1 = require("fs");
class SemanticAnalyzerAdapter {
    constructor(command = './faxc/packages/sema/bin/sema') {
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
                tempFilePath = `.temp_sema_input.json`;
                (0, fs_1.writeFileSync)(tempFilePath, JSON.stringify(input));
            }
            const command = `${this.command} "${tempFilePath}"`;
            const result = (0, child_process_1.execSync)(command, { encoding: 'utf8', timeout: 300000 });
            // Try to parse the result as JSON
            try {
                return JSON.parse(result.trim());
            }
            catch {
                // If parsing fails, return the raw result
                return result.trim();
            }
        }
        catch (error) {
            throw new Error(`Semantic analyzer execution failed: ${error.message}`);
        }
    }
    validateInput(input) {
        return typeof input === 'string' || typeof input === 'object';
    }
    getName() {
        return 'semantic-analyzer';
    }
    getVersion() {
        return '1.0.0';
    }
}
exports.SemanticAnalyzerAdapter = SemanticAnalyzerAdapter;
