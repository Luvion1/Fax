"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Pipeline = exports.PipelineBuilder = exports.PipelineExecutor = void 0;
const perf_hooks_1 = require("perf_hooks");
class PipelineExecutor {
    async executeStage(stage, input) {
        const startTime = perf_hooks_1.performance.now();
        try {
            const result = await stage.adapter.execute(input);
            const executionTime = perf_hooks_1.performance.now() - startTime;
            return {
                success: true,
                data: result,
                time: executionTime,
                warnings: []
            };
        }
        catch (error) {
            const executionTime = perf_hooks_1.performance.now() - startTime;
            return {
                success: false,
                error: error.message,
                time: executionTime,
                warnings: []
            };
        }
    }
}
exports.PipelineExecutor = PipelineExecutor;
class PipelineBuilder {
    constructor() {
        this.stages = [];
    }
    addStage(name, adapter, config = {}) {
        this.stages.push({ name, adapter, config });
        return this;
    }
    build() {
        return new Pipeline(this.stages);
    }
}
exports.PipelineBuilder = PipelineBuilder;
class Pipeline {
    constructor(stages) {
        this.stages = stages;
    }
    async execute(initialInput) {
        let currentInput = initialInput;
        for (const stage of this.stages) {
            const result = await new PipelineExecutor().executeStage(stage, currentInput);
            if (!result.success) {
                throw new Error(`Stage ${stage.name} failed: ${result.error}`);
            }
            currentInput = result.data;
        }
        return currentInput;
    }
}
exports.Pipeline = Pipeline;
