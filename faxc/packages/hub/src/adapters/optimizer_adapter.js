"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.OptimizerAdapter = void 0;
const child_process_1 = require("child_process");
const fs_1 = require("fs");
class OptimizerAdapter {
    constructor(command = './faxc/packages/optimizer/target/release/fax-opt') {
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
                tempFilePath = `.temp_optimizer_input.json`;
                (0, fs_1.writeFileSync)(tempFilePath, JSON.stringify(input));
            }
            const command = `${this.command} "${tempFilePath}" --opt-level=intermediate`;
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
            throw new Error(`Optimizer execution failed: ${error.message}`);
        }
    }
    validateInput(input) {
        return typeof input === 'string' || typeof input === 'object';
    }
    getName() {
        return 'optimizer';
    }
    getVersion() {
        return '1.0.0';
    }
}
exports.OptimizerAdapter = OptimizerAdapter;
