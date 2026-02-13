"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.LexerAdapter = void 0;
const child_process_1 = require("child_process");
const fs_1 = require("fs");
class LexerAdapter {
    constructor(command = '../lexer/target/release/lexer') {
        this.command = command;
    }
    async execute(input) {
        const tempFilePath = `.temp_lexer_input.fax`;
        try {
            if (typeof input === 'string') {
                (0, fs_1.writeFileSync)(tempFilePath, input);
            }
            else {
                (0, fs_1.writeFileSync)(tempFilePath, JSON.stringify(input));
            }
            const command = `${this.command} "${tempFilePath}"`;
            const result = (0, child_process_1.execSync)(command, { encoding: 'utf8', timeout: 300000 });
            // Cleanup temp file
            try {
                require('fs').unlinkSync(tempFilePath);
            }
            catch { }
            // Try to parse the result as JSON
            try {
                return JSON.parse(result.trim());
            }
            catch {
                return result.trim();
            }
        }
        catch (error) {
            try {
                require('fs').unlinkSync(tempFilePath);
            }
            catch { }
            throw new Error(`Lexer execution failed: ${error.message}`);
        }
    }
    validateInput(input) {
        return typeof input === 'string' || typeof input === 'object';
    }
    getName() {
        return 'lexer';
    }
    getVersion() {
        return '1.0.0';
    }
}
exports.LexerAdapter = LexerAdapter;
