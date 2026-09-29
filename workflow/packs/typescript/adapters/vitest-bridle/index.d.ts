export type StepFn = (world: Record<string, any>, ...groups: string[]) => void | Promise<void>;

export interface Steps {
  given(pattern: RegExp, fn: StepFn): void;
  when(pattern: RegExp, fn: StepFn): void;
  then(pattern: RegExp, fn: StepFn): void;
}

export function createSteps(): Steps & { defs: object };

export function registerBridleSpecs(opts: {
  steps: Steps;
  root?: string;
  capability?: string;
  scenarios?: string[];
}): Promise<void>;
