# Lessons Learned: QuickTools Tray Menu Event Debug

**Date**: 2026-04-21
**Issue**: Tray menu tool click with parameters opens dialog in hidden window instead of showing the main window first.

---

## Root Cause

The running app binary was NOT the freshly built one.

When running `cp -R .../QuickTools.app /Applications/QuickTools.app` while the app is already running, the copy silently fails because the running app holds file locks on its binary and `.app` bundle. `cp -R` doesn't overwrite a running app's contents — it just... does nothing on macOS when the target is a running app.

Result: the binary in `/Applications/QuickTools.app` stayed at the old timestamp (11:34), while the newly built binary was at 15:34. All changes were being tested against the old code.

## Key Debugging Steps

1. **Check binary timestamp** — always verify the installed binary matches the build timestamp:
   ```bash
   ls -la /Applications/QuickTools.app/Contents/MacOS/quicktools
   date -r /Applications/QuickTools.app/Contents/MacOS/quicktools "+%Y-%m-%d %H:%M:%S"
   ```

2. **Add logging early** — add logs at the earliest possible point (tray setup, event handler entry) so you can confirm code path is reached even if the log file gets overwritten.

3. **Check log file timestamp** — if logs don't appear after a restart, the log file was likely overwritten and you're reading old content:
   ```bash
   stat -f "%Sm" ~/Library/Application\ Support/QuickTools/logs/quicktools.log
   ```

## The Actual Fix

In `lib.rs`, the `on_menu_event` handler now shows the main window before emitting the `open_param_dialog` event — same logic as the "打开主界面" menu item:

```rust
.on_menu_event(|app, event| {
    let id = &event.id.0;
    if id == "quit" {
        app.exit(0);
        return;
    }
    if id == "show" || id.starts_with("tool:") {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
    if let Some(tool_id) = id.strip_prefix("tool:") {
        let payload = serde_json::json!({ "toolId": tool_id });
        let _ = app.emit("open_param_dialog", payload);
    }
})
```

## Correct Way to Update a Running macOS App

```bash
# 1. Kill the running app first
pkill -9 -f "QuickTools"

# 2. Wait for process to fully exit
sleep 2

# 3. Copy the binary directly (not the whole .app, just the binary)
cp -f ".../QuickTools.app/Contents/MacOS/quicktools" \
   "/Applications/QuickTools.app/Contents/MacOS/quicktools"

# 4. Restart
open -a QuickTools
```

Or use the full app bundle copy, but ONLY after killing the running process:
```bash
pkill -9 -f "QuickTools"; sleep 2; cp -R ".../QuickTools.app" "/Applications/QuickTools.app"
```
