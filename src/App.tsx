import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Wrench, History, Settings, RefreshCw } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";

import { ToolList, ExecutionPanel, LogViewer, SettingsPage } from "@/components/quicktools";
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
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-lg font-semibold">{t("tools.title", "Tools")}</h2>
        <Button variant="outline" size="sm" onClick={refresh}>
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
  const { t } = useTranslation();
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

  return (
    <div className="h-screen w-screen overflow-hidden bg-background">
      <Tabs defaultValue="tools" className="h-full flex flex-col">
        <div className="border-b px-4 py-2 flex items-center justify-between">
          <TabsList>
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

        <TabsContent value="tools" className="flex-1 overflow-auto p-4 m-0">
          <ToolsPage />
        </TabsContent>

        <TabsContent value="logs" className="flex-1 overflow-auto p-4 m-0">
          <LogsPage />
        </TabsContent>

        <TabsContent value="settings" className="flex-1 overflow-auto p-4 m-0">
          <SettingsContent />
        </TabsContent>
      </Tabs>
    </div>
  );
}

export default App;