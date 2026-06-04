import { useState, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Wrench, History, Settings, RefreshCw, Sparkles } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { listen } from "@tauri-apps/api/event";

import { ToolList, ExecutionPanel, LogViewer, SettingsPage, HarmonyIntegrationTool } from "@/components/quicktools";
import { ParamDialog } from "@/components/ParamDialog";
import { useTools, useExecution, useLogs } from "@/hooks/useTools";

function ToolsPage() {
  const { t } = useTranslation();
  const { tools, loading, error, refresh } = useTools();
  const { executing, result, execute, clearResult } = useExecution();

  const handleExecute = useCallback(
    async (toolId: string, params: Record<string, string>) => {
      try {
        const execResult = await execute(toolId, params);
        toast.success(
          t("tools.executed", "Tool executed: {{name}}", { name: execResult.toolName }),
          {
            description: execResult.status,
          }
        );
      } catch {
        toast.error(t("tools.executionFailed", "Execution failed"));
      }
    },
    [execute, t]
  );

  return (
    <div className="h-full flex flex-col">
      <HarmonyIntegrationTool />
      <div className="mb-5 flex items-center justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <span className="flex h-8 w-8 items-center justify-center rounded-lg border border-blue-500/15 bg-blue-500/10 text-blue-500 shadow-sm shadow-blue-500/10">
              <Sparkles className="h-4 w-4" />
            </span>
            <h2 className="text-xl font-semibold tracking-tight">
              {t("tools.title", "Tools")}
            </h2>
          </div>
          <p className="mt-1 text-xs text-muted-foreground">
            {tools.length > 0
              ? t("tools.count", {
                  count: tools.length,
                  defaultValue: "{{count}} tools ready",
                })
              : t("tools.empty", "No tools configured")}
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={refresh}
          className="h-9 rounded-lg border-border/80 bg-card/80 px-3 text-sm shadow-sm backdrop-blur hover:bg-card"
        >
          <RefreshCw className="h-4 w-4 mr-2" />
          {t("common.refresh", "Refresh")}
        </Button>
      </div>

      <ToolList
        tools={tools}
        loading={loading}
        error={error}
        onExecute={handleExecute}
        executing={executing}
        result={result}
      />

      <ExecutionPanel result={result} onClose={clearResult} />
    </div>
  );
}

function LogsPage() {
  const { logs, total, loading, query, goToPage, updateQuery, refresh } = useLogs({
    page: 1,
    pageSize: 20,
  });

  return (
    <div className="h-full">
      <LogViewer
        logs={logs}
        total={total}
        loading={loading}
        page={query.page || 1}
        pageSize={query.pageSize || 20}
        statusFilter={query.status}
        onPageChange={goToPage}
        onStatusFilterChange={(status) => updateQuery({ status, page: 1 })}
        onRefresh={refresh}
      />
    </div>
  );
}

function SettingsContent() {
  return <SettingsPage />;
}

function App() {
  const { t } = useTranslation();
  const [paramDialogToolId, setParamDialogToolId] = useState<string | null>(null);

  useEffect(() => {
    const unlisten = listen<string | { toolId?: string }>("open_param_dialog", (event) => {
      const payload = event.payload;
      const toolId =
        typeof payload === "string" ? payload : payload?.toolId;
      if (toolId) setParamDialogToolId(toolId);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <div className="h-screen w-screen overflow-hidden bg-background text-foreground">
      {paramDialogToolId && (
        <ParamDialog
          toolId={paramDialogToolId}
          onClose={() => setParamDialogToolId(null)}
        />
      )}
      <Tabs defaultValue="tools" className="h-full flex flex-col">
        <div
          className="glass-header flex items-center justify-between px-5 pb-3 pt-4"
          data-tauri-drag-region
        >
          <TabsList className="no-drag shadow-sm">
            <TabsTrigger value="tools" className="gap-2">
              <Wrench className="h-4 w-4" />
              {t("tools.title", "工具")}
            </TabsTrigger>
            <TabsTrigger value="logs" className="gap-2">
              <History className="h-4 w-4" />
              {t("logs.title", "日志")}
            </TabsTrigger>
            <TabsTrigger value="settings" className="gap-2">
              <Settings className="h-4 w-4" />
              {t("settings.title", "设置")}
            </TabsTrigger>
          </TabsList>
        </div>

        <TabsContent value="tools" className="m-0 flex-1 overflow-auto p-0">
          <main className="app-main-surface h-full p-5">
            <ToolsPage />
          </main>
        </TabsContent>

        <TabsContent value="logs" className="m-0 flex-1 overflow-auto p-0">
          <main className="app-main-surface h-full p-5">
            <LogsPage />
          </main>
        </TabsContent>

        <TabsContent value="settings" className="m-0 flex-1 overflow-auto p-0">
          <main className="app-main-surface h-full p-5">
            <SettingsContent />
          </main>
        </TabsContent>
      </Tabs>
    </div>
  );
}

export default App;
