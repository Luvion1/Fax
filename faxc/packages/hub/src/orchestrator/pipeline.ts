import { ComponentAdapter, PipelineStage, CompilationResult } from '../types/interfaces';
import { execSync } from 'child_process';
import { writeFileSync } from 'fs';
import { performance } from 'perf_hooks';

export class PipelineExecutor {
  async executeStage(stage: PipelineStage, input: any): Promise<CompilationResult> {
    const startTime = performance.now();
    
    try {
      const result = await stage.adapter.execute(input);
      
      const executionTime = performance.now() - startTime;
      
      return {
        success: true,
        data: result,
        time: executionTime,
        warnings: []
      };
    } catch (error) {
      const executionTime = performance.now() - startTime;
      
      return {
        success: false,
        error: (error as Error).message,
        time: executionTime,
        warnings: []
      };
    }
  }
}

export class PipelineBuilder {
  private stages: PipelineStage[] = [];

  addStage(name: string, adapter: ComponentAdapter, config: Record<string, any> = {}): PipelineBuilder {
    this.stages.push({ name, adapter, config });
    return this;
  }

  build(): Pipeline {
    return new Pipeline(this.stages);
  }
}

export class Pipeline {
  private stages: PipelineStage[];

  constructor(stages: PipelineStage[]) {
    this.stages = stages;
  }

  async execute(initialInput: any): Promise<any> {
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