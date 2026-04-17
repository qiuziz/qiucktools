import { useState, useEffect, useCallback } from "react";
import { loadTools, executeTool, getLogs } from "@/lib/api/tools";
import type {
  Tool,
  ExecutionResult,
  ExecutionLog,
  LogQuery,
  PaginatedLogs,
} from "@/types/tool";

// Load and cache tools
export function useTools() {
  const [tools, setTools] = useState<Tool[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await loadTools();
      setTools(data.filter((t) => t.enabled));
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load tools");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { tools, loading, error, refresh };
}

// Execute a tool
export function useExecution() {
  const [executing, setExecuting] = useState(false);
  const [result, setResult] = useState<ExecutionResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const execute = useCallback(
    async (toolId: string, params: Record<string, string> = {}) => {
      setExecuting(true);
      setError(null);
      setResult(null);
      try {
        const execResult = await executeTool(toolId, params);
        setResult(execResult);
        return execResult;
      } catch (e) {
        const msg = e instanceof Error ? e.message : "Execution failed";
        setError(msg);
        throw e;
      } finally {
        setExecuting(false);
      }
    },
    []
  );

  const clearResult = useCallback(() => {
    setResult(null);
    setError(null);
  }, []);

  return { executing, result, error, execute, clearResult };
}

// Query execution logs
export function useLogs(initialQuery: LogQuery = {}) {
  const [logs, setLogs] = useState<ExecutionLog[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [query, setQuery] = useState<LogQuery>(initialQuery);

  const fetchLogs = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await getLogs(query);
      setLogs(data.logs);
      setTotal(data.total);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load logs");
    } finally {
      setLoading(false);
    }
  }, [query]);

  useEffect(() => {
    fetchLogs();
  }, [fetchLogs]);

  const updateQuery = useCallback((updates: Partial<LogQuery>) => {
    setQuery((prev) => ({ ...prev, ...updates }));
  }, []);

  const goToPage = useCallback(
    (page: number) => {
      setQuery((prev) => ({ ...prev, page }));
    },
    []
  );

  return {
    logs,
    total,
    loading,
    error,
    query,
    updateQuery,
    goToPage,
    refresh: fetchLogs,
  };
}