import { useState } from "react";
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
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
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

const iconMap: Record<string, React.ComponentType<{ className?: string }>> = {
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
    <Card className="hover:shadow-md transition-shadow">
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-primary/10">
              <IconComponent className="h-5 w-5 text-primary" />
            </div>
            <div>
              <CardTitle className="text-base">{tool.name}</CardTitle>
              {tool.description && (
                <CardDescription className="text-xs mt-1">
                  {tool.description}
                </CardDescription>
              )}
            </div>
          </div>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="icon" className="h-8 w-8">
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
      <CardContent className="pt-0">
        {hasParams && (
          <div className="mb-3 space-y-2">
            {tool.params.map((param) => (
              <div key={param.name}>
                <label className="text-xs text-muted-foreground">
                  {param.label}
                  {param.required && " *"}
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
        <div className="flex items-center justify-between">
          <span className="text-xs text-muted-foreground capitalize">
            {tool.type}
          </span>
          <Button
            size="sm"
            onClick={handleExecute}
            disabled={isRunning || isMissingRequired}
          >
            {isRunning ? (
              <Loader2 className="h-4 w-4 animate-spin mr-2" />
            ) : (
              <Play className="h-4 w-4 mr-2" />
            )}
            {t("tools.execute")}
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
      className="w-full mt-1 px-3 py-2 text-sm border rounded-md bg-background"
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
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
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
