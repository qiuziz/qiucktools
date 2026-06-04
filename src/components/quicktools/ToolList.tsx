import { useState, type ComponentType } from "react";
import { useTranslation } from "react-i18next";
import {
  Play,
  Loader2,
  Terminal,
  FileCode,
  FolderOpen,
  Bell,
  MoreVertical,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { Tool, ToolParam, ExecutionResult } from "@/types/tool";

const iconMap: Record<string, ComponentType<{ className?: string }>> = {
  terminal: Terminal,
  "file-code": FileCode,
  "folder-open": FolderOpen,
  bell: Bell,
};

interface ToolCardProps {
  tool: Tool;
  onExecute: (toolId: string, params: Record<string, string>) => void;
  executing: boolean;
  result?: ExecutionResult | null;
}

export function ToolCard({ tool, onExecute, executing, result }: ToolCardProps) {
  const { t } = useTranslation();
  const [params, setParams] = useState<Record<string, string>>({});

  const IconComponent = iconMap[tool.icon] || Terminal;

  const handleExecute = () => {
    onExecute(tool.id, getEffectiveParams(tool, params));
  };

  const hasParams = tool.params.length > 0;
  const isRunning = executing && result?.toolId === tool.id;
  const isMissingRequired = tool.params.some((param) => {
    if (!param.required) return false;
    return !getEffectiveParams(tool, params)[param.name];
  });

  return (
    <Card className="group flex min-h-[172px] flex-col overflow-hidden border-border/80 bg-card/85 shadow-[0_10px_30px_-24px_rgba(15,23,42,0.7)] backdrop-blur transition-all duration-200 hover:-translate-y-0.5 hover:border-blue-500/30 hover:bg-card hover:shadow-[0_20px_45px_-28px_rgba(10,132,255,0.55)] dark:bg-card/75">
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between">
          <div className="min-w-0 flex-1 flex items-start gap-3">
            <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl border border-blue-500/10 bg-gradient-to-br from-blue-500/15 to-blue-500/5 text-blue-500 shadow-inner shadow-white/60 dark:shadow-black/20">
              <IconComponent className="h-5 w-5 text-primary" />
            </div>
            <div className="min-w-0 pt-0.5">
              <CardTitle className="truncate text-base font-semibold tracking-tight">
                {tool.name}
              </CardTitle>
              {tool.description && (
                <CardDescription className="mt-1 line-clamp-2 text-sm leading-5">
                  {tool.description}
                </CardDescription>
              )}
            </div>
          </div>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="h-8 w-8 shrink-0 rounded-lg text-muted-foreground opacity-70 hover:opacity-100"
              >
                <MoreVertical className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem>{t("common.edit")}</DropdownMenuItem>
              <DropdownMenuItem className="text-destructive">
                {t("common.delete")}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </CardHeader>
      <CardContent className="flex flex-1 flex-col pt-0">
        {hasParams && (
          <div className="mb-4 grid gap-3">
            {tool.params.map((param) => (
              <div key={param.name}>
                <label className="text-xs font-medium text-muted-foreground">
                  {param.label}
                  {param.required && (
                    <span className="ml-0.5 text-blue-500">*</span>
                  )}
                </label>
                <ToolParamField
                  param={param}
                  value={params[param.name] ?? getDefaultParamValue(param)}
                  onChange={(value) =>
                    setParams((prev) => ({ ...prev, [param.name]: value }))
                  }
                />
              </div>
            ))}
          </div>
        )}
        <div className="mt-auto flex items-center justify-between gap-3 border-t border-border/70 pt-3">
          <span className="inline-flex h-7 items-center rounded-md bg-muted/70 px-2 text-xs font-medium capitalize text-muted-foreground">
            {tool.type}
          </span>
          <Button
            size="sm"
            onClick={handleExecute}
            disabled={isRunning || isMissingRequired}
            className="h-9 min-w-[116px] rounded-lg px-4 text-sm shadow-sm shadow-blue-500/20"
          >
            {isRunning ? (
              <Loader2 className="h-4 w-4 animate-spin mr-2" />
            ) : (
              <Play className="h-4 w-4 mr-2" />
            )}
            {t("tools.execute", "Run")}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

interface ToolParamFieldProps {
  param: ToolParam;
  value: string;
  onChange: (value: string) => void;
}

function ToolParamField({ param, value, onChange }: ToolParamFieldProps) {
  if (param.type === "select") {
    return (
      <Select value={value} onValueChange={onChange}>
        <SelectTrigger className="mt-1">
          <SelectValue placeholder={param.label} />
        </SelectTrigger>
        <SelectContent>
          {param.options?.map((opt) => (
            <SelectItem key={opt.value} value={opt.value}>
              {opt.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    );
  }

  return (
    <input
      type={param.type === "number" ? "number" : "text"}
      className="mt-1 h-10 w-full rounded-lg border border-input bg-background/80 px-3 text-sm outline-none transition-colors placeholder:text-muted-foreground/55 focus:border-blue-500/70 focus:ring-2 focus:ring-blue-500/15"
      value={value}
      onChange={(e) => onChange(e.target.value)}
      placeholder={param.label}
    />
  );
}

function getDefaultParamValue(param: ToolParam): string {
  if (param.default !== undefined && param.default !== null) {
    return String(param.default);
  }
  return "";
}

function getDefaultParams(tool: Tool): Record<string, string> {
  return tool.params.reduce<Record<string, string>>((defaults, param) => {
    const defaultValue = getDefaultParamValue(param);
    if (defaultValue) {
      defaults[param.name] = defaultValue;
    }
    return defaults;
  }, {});
}

function getEffectiveParams(
  tool: Tool,
  params: Record<string, string>
): Record<string, string> {
  return { ...getDefaultParams(tool), ...params };
}

interface ToolListProps {
  tools: Tool[];
  loading: boolean;
  error?: string | null;
  onExecute: (toolId: string, params: Record<string, string>) => void;
  executing: boolean;
  result?: ExecutionResult | null;
}

export function ToolList({
  tools,
  loading,
  error,
  onExecute,
  executing,
  result,
}: ToolListProps) {
  const { t } = useTranslation();

  if (loading) {
    return (
      <div className="flex items-center justify-center py-20">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="text-center py-20 text-destructive">
        <p>{error}</p>
      </div>
    );
  }

  if (tools.length === 0) {
    return (
      <div className="text-center py-20 text-muted-foreground">
        <Terminal className="h-12 w-12 mx-auto mb-4 opacity-50" />
        <p>{t("tools.empty", "No tools configured")}</p>
        <p className="text-sm mt-2">{t("tools.emptyHint", "Add tools to ~/work/quicktools/tools.json")}</p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 gap-4 xl:grid-cols-2 2xl:grid-cols-3">
      {tools.map((tool) => (
        <ToolCard
          key={tool.id}
          tool={tool}
          onExecute={onExecute}
          executing={executing}
          result={result}
        />
      ))}
    </div>
  );
}
