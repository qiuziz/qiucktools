import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Wrench, History, Settings } from "lucide-react";

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

        <TabsContent value="tools" className="flex-1 overflow-auto p-4">
          <div className="text-muted-foreground text-sm text-center mt-20">
            工具管理 — Phase 4 实现
          </div>
        </TabsContent>

        <TabsContent value="logs" className="flex-1 overflow-auto p-4">
          <div className="text-muted-foreground text-sm text-center mt-20">
            执行日志 — Phase 4 实现
          </div>
        </TabsContent>

        <TabsContent value="settings" className="flex-1 overflow-auto p-4">
          <div className="text-muted-foreground text-sm text-center mt-20">
            设置页面 — Phase 4 实现
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}

export default App;
