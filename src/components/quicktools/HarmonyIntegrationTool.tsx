import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, GitBranch, Loader2, CheckCircle2 } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { useTranslation } from "react-i18next";

interface BranchPreview {
  newBranch: string;
  currentBranch: string;
  exists: boolean;
}

interface IntegrationResult {
  success: boolean;
  mrUrl: string | null;
  newBranch: string;
  message: string;
}

export function HarmonyIntegrationTool() {
  const { t } = useTranslation();
  const [inputValue, setInputValue] = useState("");
  const [loading, setLoading] = useState(false);
  const [preview, setPreview] = useState<BranchPreview | null>(null);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [integrating, setIntegrating] = useState(false);
  const [result, setResult] = useState<IntegrationResult | null>(null);

  const handlePreview = async () => {
    const value = inputValue.trim();
    if (!value) {
      toast.error("请输入新版本分支名");
      return;
    }
    if (!value.startsWith("release-")) {
      toast.error("分支名必须以 release- 开头");
      return;
    }

    setLoading(true);
    try {
      const preview = await invoke<BranchPreview>("harmony_branch_preview", {
        newBranch: value,
      });
      setPreview(preview);
      setConfirmOpen(true);
    } catch (err) {
      toast.error(String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleConfirm = async () => {
    if (!preview) return;
    setConfirmOpen(false);
    setIntegrating(true);

    try {
      const res = await invoke<IntegrationResult>("harmony_branch_integrate", {
        newBranch: preview.newBranch,
        currentBranch: preview.currentBranch,
      });
      setResult(res);
      toast.success("集成分支完成", {
        description: res.message,
      });
    } catch (err) {
      toast.error("集成分支失败", {
        description: String(err),
      });
    } finally {
      setIntegrating(false);
    }
  };

  const handleReset = () => {
    setInputValue("");
    setPreview(null);
    setResult(null);
    setConfirmOpen(false);
  };

  if (result) {
    return (
      <div className="rounded-xl border border-border/80 bg-card/85 p-6 shadow-[0_10px_30px_-24px_rgba(15,23,42,0.7)] backdrop-blur">
        <div className="flex items-start gap-4">
          <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl border border-green-500/10 bg-green-500/10 text-green-500">
            <CheckCircle2 className="h-6 w-6" />
          </div>
          <div className="flex-1 min-w-0">
            <h3 className="text-lg font-semibold">鸿蒙新版本集成分支</h3>
            <p className="mt-1 text-sm text-muted-foreground">{result.message}</p>
            {result.mrUrl && (
              <a
                href={result.mrUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="mt-2 inline-flex items-center gap-1 text-sm text-blue-500 hover:underline"
              >
                查看 MR
                <ArrowRight className="h-3 w-3" />
              </a>
            )}
            <div className="mt-4 flex items-center gap-3 rounded-lg bg-muted/50 p-3">
              <GitBranch className="h-4 w-4 text-muted-foreground" />
              <code className="text-sm font-mono">{result.newBranch}</code>
            </div>
            <Button variant="outline" size="sm" onClick={handleReset} className="mt-4">
              继续下一个版本
            </Button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="rounded-xl border border-border/80 bg-card/85 p-6 shadow-[0_10px_30px_-24px_rgba(15,23,42,0.7)] backdrop-blur">
      <div className="mb-4 flex items-center gap-3">
        <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl border border-blue-500/10 bg-blue-500/5 text-blue-500 shadow-inner shadow-white/60">
          <GitBranch className="h-5 w-5" />
        </div>
        <div>
          <h3 className="text-base font-semibold tracking-tight">鸿蒙新版本集成分支</h3>
          <p className="text-xs text-muted-foreground">
            输入新版本分支名，自动推导当前分支，创建 MR 并合并，从 master 创建新分支
          </p>
        </div>
      </div>

      <div className="space-y-4">
        <div>
          <label className="text-sm font-medium">
            新版本分支
            <span className="ml-1 text-red-500">*</span>
          </label>
          <div className="mt-1.5 flex gap-2">
            <Input
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value)}
              placeholder="release-17.37"
              className="flex-1"
              onKeyDown={(e) => {
                if (e.key === "Enter") handlePreview();
              }}
            />
            <Button onClick={handlePreview} disabled={loading || !inputValue.trim()}>
              {loading ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                "预览"
              )}
            </Button>
          </div>
          <p className="mt-1.5 text-xs text-muted-foreground">
            格式：release-x.xx 或 release-x.xx.xx
          </p>
        </div>
      </div>

      {/* 确认弹窗 */}
      <Dialog open={confirmOpen} onOpenChange={(open) => !open && setConfirmOpen(false)}>
        <DialogContent className="max-w-sm">
          <DialogHeader>
            <DialogTitle>确认分支信息</DialogTitle>
            <DialogDescription className="space-y-2 pt-2">
              <p>即将执行以下操作：</p>
              <div className="rounded-lg border bg-muted/50 p-3 font-mono text-sm">
                <div className="flex items-center gap-2">
                  <span className="text-muted-foreground">当前分支:</span>
                  <span className="text-blue-500">{preview?.currentBranch}</span>
                </div>
                <div className="flex items-center justify-center py-1">
                  <ArrowRight className="h-4 w-4 text-muted-foreground" />
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-muted-foreground">新分支:</span>
                  <span className="text-green-500">{preview?.newBranch}</span>
                </div>
              </div>
              <div className="space-y-1 text-xs text-muted-foreground">
                <p>1. 创建 MR：当前分支 → master</p>
                <p>2. 自动合并该 MR</p>
                <p>3. 从 master 创建新分支</p>
              </div>
            </DialogDescription>
          </DialogHeader>
          <DialogFooter className="gap-2 sm:gap-0">
            <Button variant="outline" onClick={() => setConfirmOpen(false)}>
              取消
            </Button>
            <Button onClick={handleConfirm} disabled={integrating}>
              {integrating ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  执行中...
                </>
              ) : (
                "确认执行"
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}