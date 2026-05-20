import { useEffect, useState } from "react";
import { loadTools, executeTool } from "@/lib/api/tools";
import type { Tool, ToolParam } from "@/types/tool";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import { AlertCircle } from "lucide-react";

interface ParamDialogProps {
  toolId: string;
  onClose: () => void;
}

interface PathError {
  field: string;
  message: string;
}

export function ParamDialog({ toolId, onClose }: ParamDialogProps) {
  const { t } = useTranslation();
  const [tool, setTool] = useState<Tool | null>(null);
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [formValues, setFormValues] = useState<Record<string, string>>({});
  const [pathError, setPathError] = useState<PathError | null>(null);

  useEffect(() => {
    async function fetchTool() {
      setLoading(true);
      try {
        const tools = await loadTools();
        const found = tools.find((t) => t.id === toolId);
        setTool(found ?? null);

        if (found) {
          const defaults: Record<string, string> = {};
          for (const param of found.params) {
            if (param.default !== undefined && param.default !== null) {
              defaults[param.name] = String(param.default);
            }
          }
          const cleaned: Record<string, string> = {};
          for (const [k, v] of Object.entries(defaults)) {
            cleaned[k] = cleanValue(v);
          }
          setFormValues(cleaned);
        }
      } catch {
        toast.error("Failed to load tool");
        onClose();
      } finally {
        setLoading(false);
      }
    }
    fetchTool();
  }, [toolId]);

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!tool) return;

    const missingRequired = tool.params.filter(
      (p) => p.required && !formValues[p.name]
    );
    if (missingRequired.length > 0) {
      toast.error(
        `请填写必填字段: ${missingRequired.map((p) => p.label).join(", ")}`
      );
      return;
    }

    setSubmitting(true);
    try {
      await executeTool(tool.id, formValues);
      toast.success(
        t("tools.executed", "Tool executed: {{name}}", { name: tool.name }),
        { description: tool.description ?? undefined }
      );
      onClose();
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);

      // 检查是否是路径相关的错误
      if (isPathError(errorMsg)) {
        const field = detectPathField(errorMsg, tool.params);
        setPathError({ field, message: extractPathErrorMessage(errorMsg) });
      } else {
        // 对于其他错误，仍然显示 toast
        toast.error(t("tools.executionFailed", "执行失败"), {
          description: errorMsg,
        });
      }
    } finally {
      setSubmitting(false);
    }
  };

  const handleValueChange = (name: string, value: string) => {
    setFormValues((prev) => ({ ...prev, [name]: cleanValue(value) }));
    // 清除路径错误当用户修改值时
    if (pathError?.field === name) {
      setPathError(null);
    }
  };

  const cleanValue = (v: string) => {
    const trimmed = v.trim();
    if (trimmed.startsWith("'") && trimmed.endsWith("'") && trimmed.length >= 2) {
      return trimmed.slice(1, -1);
    }
    return v;
  };

  // 检查错误信息是否与路径相关
  const isPathError = (msg: string): boolean => {
    const pathErrorPatterns = [
      /文件不存在|file.*not.*found|not found.*file/i,
      /源文件不存在/i,
      /path.*not.*exist/i,
      /no such file/i,
    ];
    return pathErrorPatterns.some((p) => p.test(msg));
  };

  // 从错误信息中检测是哪个字段的问题
  const detectPathField = (msg: string, params: ToolParam[]): string => {
    // 尝试从错误信息中提取路径
    const pathMatch = msg.match(/['"]?(\/[^'"\n]+)['"]?/);
    if (pathMatch) {
      const errorPath = pathMatch[1];
      // 找到包含该路径的字段
      for (const param of params) {
        const value = formValues[param.name];
        if (value && (value.includes(errorPath) || errorPath.includes(value))) {
          return param.label || param.name;
        }
      }
    }
    // 默认返回第一个文本输入字段
    const textParam = params.find((p) => p.type === "text");
    return textParam?.label || params[0]?.label || "参数";
  };

  // 提取路径错误的人类可读信息
  const extractPathErrorMessage = (msg: string): string => {
    // 移除命令输出中的冗余部分，只保留核心错误信息
    if (msg.includes("源文件不存在")) {
      const pathMatch = msg.match(/源文件不存在[：:]\s*(.+)/);
      return pathMatch ? `文件未找到: ${pathMatch[1]}` : "文件路径无效或文件不存在";
    }
    if (msg.includes("file not found")) {
      return "指定的文件不存在，请检查路径是否正确";
    }
    if (msg.includes("no such file")) {
      return "路径无效或文件不存在";
    }
    return msg;
  };

  if (loading || !tool) {
    return (
      <Dialog open onOpenChange={(open) => !open && onClose()}>
        <DialogContent className="max-w-md">
          <div className="py-8 text-center text-muted-foreground">
            {loading ? "Loading..." : "Tool not found"}
          </div>
        </DialogContent>
      </Dialog>
    );
  }

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="max-w-md overflow-hidden">
        <form onSubmit={handleSubmit} className="flex min-h-0 flex-col">
          <DialogHeader>
            <DialogTitle>{tool.name}</DialogTitle>
            {tool.description && (
              <DialogDescription>{tool.description}</DialogDescription>
            )}
          </DialogHeader>

          <div className="space-y-4 px-6 py-5">
            {tool.params.length === 0 && (
              <p className="text-sm text-muted-foreground">
                No parameters required.
              </p>
            )}
            {tool.params.map((param) => (
              <ParamField
                key={param.name}
                param={param}
                value={formValues[param.name] ?? ""}
                onChange={(val) => handleValueChange(param.name, val)}
                error={pathError?.field === (param.label || param.name) ? pathError.message : undefined}
              />
            ))}
          </div>

          <DialogFooter>
            <Button
              type="button"
              variant="secondary"
              onClick={onClose}
              disabled={submitting}
            >
              {t("common.cancel", "Cancel")}
            </Button>
            <Button type="submit" disabled={submitting}>
              {submitting
                ? t("common.executing", "Executing...")
                : t("common.confirm", "Confirm")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

interface ParamFieldProps {
  param: ToolParam;
  value: string;
  onChange: (value: string) => void;
  error?: string;
}

function ParamField({ param, value, onChange, error }: ParamFieldProps) {
  const label = (
    <label className="text-sm font-medium">
      {param.label}
      {param.required && (
        <span className="text-red-500 ml-1" aria-label="required">
          *
        </span>
      )}
    </label>
  );

  if (param.type === "select") {
    return (
      <div className="space-y-1.5">
        {label}
        <Select value={value} onValueChange={onChange}>
          <SelectTrigger>
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
      </div>
    );
  }

  if (param.type === "number") {
    return (
      <div className="space-y-1.5">
        {label}
        <Input
          type="number"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          min={param.min}
          max={param.max}
          placeholder={param.label}
        />
        {error && (
          <div className="flex items-center gap-1 text-sm text-red-500">
            <AlertCircle className="h-3 w-3" />
            <span>{error}</span>
          </div>
        )}
      </div>
    );
  }

  // Default: text with optional error
  return (
    <div className="space-y-1.5">
      <div className="flex items-center gap-2">
        {label}
        {error && (
          <div className="flex items-center gap-1 text-sm text-red-500">
            <AlertCircle className="h-3 w-3" />
          </div>
        )}
      </div>
      <Input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={param.label}
        className={error ? "border-red-500 focus:border-red-500" : ""}
      />
      {error && (
        <div className="flex items-start gap-1 text-sm text-red-500">
          <AlertCircle className="h-3 w-3 mt-0.5 shrink-0" />
          <span>{error}</span>
        </div>
      )}
    </div>
  );
}
