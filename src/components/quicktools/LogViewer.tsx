import { useTranslation } from "react-i18next";
import {
  CheckCircle2,
  XCircle,
  Clock,
  ChevronLeft,
  ChevronRight,
  RefreshCw,
  Filter,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import type { ExecutionLog, ExecutionStatus } from "@/types/tool";

interface LogViewerProps {
  logs: ExecutionLog[];
  total: number;
  loading: boolean;
  page: number;
  pageSize: number;
  statusFilter?: string;
  onPageChange: (page: number) => void;
  onStatusFilterChange: (status: string | undefined) => void;
  onRefresh: () => void;
}

const statusConfig: Record<
  ExecutionStatus,
  { icon: React.ComponentType<{ className?: string }>; color: string; variant: "default" | "destructive" | "secondary" }
> = {
  success: { icon: CheckCircle2, color: "text-green-500", variant: "secondary" },
  failed: { icon: XCircle, color: "text-red-500", variant: "destructive" },
  timeout: { icon: Clock, color: "text-yellow-500", variant: "secondary" },
};

export function LogViewer({
  logs,
  total,
  loading,
  page,
  pageSize,
  statusFilter,
  onPageChange,
  onStatusFilterChange,
  onRefresh,
}: LogViewerProps) {
  const { t } = useTranslation();

  const totalPages = Math.ceil(total / pageSize);

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleString();
  };

  const formatDuration = (ms: number) => {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
  };

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-base flex items-center gap-2">
            {t("logs.title", "Execution Logs")}
            <Badge variant="outline">{total}</Badge>
          </CardTitle>
          <div className="flex items-center gap-2">
            <Select
              value={statusFilter || "all"}
              onValueChange={(val) =>
                onStatusFilterChange(val === "all" ? undefined : val)
              }
            >
              <SelectTrigger className="w-32">
                <Filter className="h-4 w-4 mr-2" />
                <SelectValue placeholder={t("logs.filter", "Filter")} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">{t("logs.all", "All")}</SelectItem>
                <SelectItem value="success">{t("logs.success", "Success")}</SelectItem>
                <SelectItem value="failed">{t("logs.failed", "Failed")}</SelectItem>
                <SelectItem value="timeout">{t("logs.timeout", "Timeout")}</SelectItem>
              </SelectContent>
            </Select>
            <Button variant="outline" size="icon" onClick={onRefresh}>
              <RefreshCw className={`h-4 w-4 ${loading ? "animate-spin" : ""}`} />
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        {loading ? (
          <div className="flex items-center justify-center py-10">
            <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : logs.length === 0 ? (
          <div className="text-center py-10 text-muted-foreground">
            <p>{t("logs.empty", "No execution logs")}</p>
          </div>
        ) : (
          <>
            <div className="space-y-2">
              {logs.map((log) => {
                const config = statusConfig[log.status as ExecutionStatus] || statusConfig.failed;
                const StatusIcon = config.icon;

                return (
                  <div
                    key={log.id}
                    className="flex items-center justify-between p-3 rounded-lg border bg-card hover:bg-muted/50 transition-colors"
                  >
                    <div className="flex items-center gap-3">
                      <StatusIcon className={`h-5 w-5 ${config.color}`} />
                      <div>
                        <p className="font-medium text-sm">{log.toolName}</p>
                        <p className="text-xs text-muted-foreground">
                          {formatDate(log.executedAt)}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-4 text-sm text-muted-foreground">
                      <span>{formatDuration(log.durationMs)}</span>
                      {log.exitCode !== null && (
                        <Badge variant={config.variant}>exit {log.exitCode}</Badge>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>

            {totalPages > 1 && (
              <div className="flex items-center justify-between mt-4 pt-4 border-t">
                <p className="text-sm text-muted-foreground">
                  {t("logs.pageInfo", "Page {{page}} of {{total}}", { page, total: totalPages })}
                </p>
                <div className="flex items-center gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => onPageChange(page - 1)}
                    disabled={page <= 1}
                  >
                    <ChevronLeft className="h-4 w-4" />
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => onPageChange(page + 1)}
                    disabled={page >= totalPages}
                  >
                    <ChevronRight className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            )}
          </>
        )}
      </CardContent>
    </Card>
  );
}