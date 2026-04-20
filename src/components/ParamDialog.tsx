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

interface ParamDialogProps {
  toolId: string;
  onClose: () => void;
}

export function ParamDialog({ toolId, onClose }: ParamDialogProps) {
  const { t } = useTranslation();
  const [tool, setTool] = useState<Tool | null>(null);
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [formValues, setFormValues] = useState<Record<string, string>>({});

  useEffect(() => {
    async function fetchTool() {
      setLoading(true);
      try {
        const tools = await loadTools();
        const found = tools.find((t) => t.id === toolId);
        setTool(found ?? null);

        // Initialise form values with defaults
        if (found) {
          const defaults: Record<string, string> = {};
          for (const param of found.params) {
            if (param.default !== undefined && param.default !== null) {
              defaults[param.name] = String(param.default);
            }
          }
          setFormValues(defaults);
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
    } catch {
      toast.error(t("tools.executionFailed", "Execution failed"));
    } finally {
      setSubmitting(false);
    }
  };

  const handleValueChange = (name: string, value: string) => {
    setFormValues((prev) => ({ ...prev, [name]: value }));
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
      <DialogContent className="max-w-md">
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>{tool.name}</DialogTitle>
            {tool.description && (
              <DialogDescription>{tool.description}</DialogDescription>
            )}
          </DialogHeader>

          <div className="space-y-4 py-4">
            {tool.params.length === 0 && (
              <p className="text-sm text-muted-foreground">No parameters required.</p>
            )}
            {tool.params.map((param) => (
              <ParamField
                key={param.name}
                param={param}
                value={formValues[param.name] ?? ""}
                onChange={(val) => handleValueChange(param.name, val)}
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
}

function ParamField({ param, value, onChange }: ParamFieldProps) {
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
      </div>
    );
  }

  // Default: text
  return (
    <div className="space-y-1.5">
      {label}
      <Input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={param.label}
      />
    </div>
  );
}
