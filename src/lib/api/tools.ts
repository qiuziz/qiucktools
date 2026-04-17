import { invoke } from "@tauri-apps/api/core";
import type { Tool, ExecutionResult, ExecutionLog, LogQuery, PaginatedLogs } from "@/types/tool";

// Tool API
export async function loadTools(): Promise<Tool[]> {
  return invoke<Tool[]>("load_tools");
}

export async function executeTool(
  toolId: string,
  params: Record<string, string>
): Promise<ExecutionResult> {
  return invoke<ExecutionResult>("execute_tool", { toolId, params });
}

// Logs API
export async function getLogs(query: LogQuery): Promise<PaginatedLogs> {
  const result = await invoke<[ExecutionLog[], number]>("get_logs", {
    toolId: query.toolId,
    status: query.status,
    from: query.from,
    to: query.to,
    page: query.page ?? 1,
    pageSize: query.pageSize ?? 20,
  });
  return {
    logs: result[0],
    total: result[1],
  };
}

// Settings API (from existing code)
export async function getSettings(): Promise<Record<string, unknown>> {
  return invoke("get_settings");
}

export async function updateSettings(
  settings: Record<string, unknown>
): Promise<void> {
  return invoke("update_settings", { settings });
}