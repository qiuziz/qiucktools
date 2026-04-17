import { useTranslation } from "react-i18next";
import {
  CheckCircle2,
  XCircle,
  Clock,
  Terminal,
  Copy,
  Check,
  ExternalLink,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useState } from "react";
import type { ExecutionResult, ExecutionStatus } from "@/types/tool";

interface ExecutionPanelProps {
  result: ExecutionResult | null;
  onClose: () => void;
}

const statusConfig: Record<
  ExecutionStatus,
  { icon: React.ComponentType<{ className?: string }>; color: string; label: string }
> = {
  success: { icon: CheckCircle2, color: "text-green-500", label: "Success" },
  failed: { icon: XCircle, color: "text-red-500", label: "Failed" },
  timeout: { icon: Clock, color: "text-yellow-500", label: "Timeout" },
};

export function ExecutionPanel({ result, onClose }: ExecutionPanelProps) {
  const { t } = useTranslation();
  const [copied, setCopied] = useState<string | null>(null);

  if (!result) return null;

  const config = statusConfig[result.status as ExecutionStatus] || statusConfig.failed;
  const StatusIcon = config.icon;

  const copyToClipboard = async (text: string, field: string) => {
    await navigator.clipboard.writeText(text);
    setCopied(field);
    setTimeout(() => setCopied(null), 2000);
  };

  const formatDuration = (ms: number) => {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
  };

  return (
    <Card className="mt-4 border-2">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <StatusIcon className={`h-5 w-5 ${config.color}`} />
            <CardTitle className="text-base">{result.toolName}</CardTitle>
            <span className={`text-xs px-2 py-1 rounded ${config.color} bg-current/10`}>
              {config.label}
            </span>
          </div>
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <span>{formatDuration(result.duration)}</span>
            {result.exitCode !== null && (
              <span>exit {result.exitCode}</span>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {result.error && (
          <div className="p-3 rounded-lg bg-destructive/10 border border-destructive/20">
            <p className="text-sm text-destructive font-medium">Error</p>
            <p className="text-sm text-destructive/80 mt-1">{result.error}</p>
          </div>
        )}

        {result.stdout && (
          <div>
            <div className="flex items-center justify-between mb-2">
              <p className="text-sm font-medium flex items-center gap-2">
                <Terminal className="h-4 w-4" />
                stdout
              </p>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => copyToClipboard(result.stdout, "stdout")}
              >
                {copied === "stdout" ? (
                  <Check className="h-4 w-4" />
                ) : (
                  <Copy className="h-4 w-4" />
                )}
              </Button>
            </div>
            <ScrollArea className="h-32 rounded border bg-muted/50 p-3">
              <pre className="text-xs font-mono whitespace-pre-wrap">
                {result.stdout}
              </pre>
            </ScrollArea>
          </div>
        )}

        {result.stderr && (
          <div>
            <div className="flex items-center justify-between mb-2">
              <p className="text-sm font-medium flex items-center gap-2 text-destructive">
                <Terminal className="h-4 w-4" />
                stderr
              </p>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => copyToClipboard(result.stderr, "stderr")}
              >
                {copied === "stderr" ? (
                  <Check className="h-4 w-4" />
                ) : (
                  <Copy className="h-4 w-4" />
                )}
              </Button>
            </div>
            <ScrollArea className="h-32 rounded border bg-destructive/10 p-3">
              <pre className="text-xs font-mono whitespace-pre-wrap text-destructive/80">
                {result.stderr}
              </pre>
            </ScrollArea>
          </div>
        )}

        <div className="flex justify-end gap-2">
          <Button variant="outline" size="sm" onClick={onClose}>
            {t("common.close")}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}