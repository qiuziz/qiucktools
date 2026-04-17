// Tool types matching Rust backend
export interface Tool {
  id: string;
  name: string;
  icon: string;
  description: string | null;
  type: ToolType;
  command: string;
  workingDir: string;
  timeoutMs: number;
  params: ToolParam[];
  sortOrder: number;
  enabled: boolean;
}

export type ToolType = 'shell' | 'script' | 'open' | 'notification';

export interface ToolParam {
  name: string;
  label: string;
  type: string;
  required?: boolean;
  default?: unknown;
  options?: ToolParamOption[];
  min?: number;
  max?: number;
}

export interface ToolParamOption {
  value: string;
  label: string;
}

// Execution types
export interface ExecutionResult {
  id: string;
  toolId: string;
  toolName: string;
  status: ExecutionStatus;
  duration: number;
  exitCode: number | null;
  stdout: string;
  stderr: string;
  error: string | null;
  params: Record<string, string>;
}

export type ExecutionStatus = 'success' | 'failed' | 'timeout';

// Log types
export interface ExecutionLog {
  id: string;
  toolId: string;
  toolName: string;
  params: string;
  status: ExecutionStatus;
  durationMs: number;
  exitCode: number | null;
  stdout: string;
  stderr: string;
  error: string | null;
  executedAt: string;
}

export interface LogQuery {
  toolId?: string;
  status?: string;
  from?: string;
  to?: string;
  page?: number;
  pageSize?: number;
}

export interface PaginatedLogs {
  logs: ExecutionLog[];
  total: number;
}