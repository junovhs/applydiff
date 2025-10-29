/**
 * tauri-bridge.js
 * All Tauri IPC calls. Listens for UI events, calls the backend, and emits results.
 * This version is simplified to remove all complex session state management.
 */
(function () {
  'use strict';

  const { invoke } = window.__TAURI__.core;

  // --- Listen for UI requests and call the backend ---

  // User wants to select a project directory
  onAppEvent('select-project-requested', async () => {
    try {
      logToConsole('📁 Requesting project directory...');
      const projectRoot = await invoke('pick_project');
      if (projectRoot) {
        window.AppState.projectRoot = projectRoot;
        emitAppEvent('project-loaded', { path: projectRoot });
      }
    } catch (e) {
      logToConsole(`❌ Project selection failed: ${e}`, 'error');
    }
  });

  // User wants the static AI prompt
  onAppEvent('copy-prompt-requested', async () => {
    try {
      const prompt = await invoke('get_ai_prompt');
      await window.__TAURI__.clipboardManager.writeText(prompt);
      logToConsole('📋 AI prompt copied to clipboard.');
    } catch (e) {
      logToConsole(`❌ Failed to copy prompt: ${e}`, 'error');
    }
  });

  // A patch has been entered and needs a preview
  onAppEvent('preview-requested', async (e) => {
    const { patch } = e.detail;
    if (!window.AppState.projectRoot || !patch || window.AppState.ui.isPreviewInFlight) {
      return;
    }

    window.AppState.ui.isPreviewInFlight = true;
    setStatus('previewing…', 'warn');
    try {
      // The backend command now directly expects the `patch` argument
      const result = await invoke('preview_patch', { patch });
      
      const hasDiff = result && result.diff && result.diff.trim();
      const hasError = result && /❌/.test(result.log || '');

      emitAppEvent('preview-ready', {
        diff: result.diff || '',
        log: result.log || '',
        hasDiff,
        hasError,
      });

      if (!hasDiff && !hasError) {
        setStatus('ready');
      } else if (hasError) {
        setStatus('preview error', 'err');
      } else {
        setStatus('ready to apply', 'ok');
      }

    } catch (e) {
      logToConsole(`❌ Preview failed: ${e}`, 'error');
      setStatus('preview error', 'err');
      emitAppEvent('preview-ready', { diff: '', log: e, hasDiff: false, hasError: true });
    } finally {
      window.AppState.ui.isPreviewInFlight = false;
    }
  });

  // User clicked "Apply Patch"
  onAppEvent('apply-requested', async (e) => {
    const { patch } = e.detail;
    if (!window.AppState.projectRoot || !patch) {
      return;
    }

    setStatus('applying…', 'warn');
    try {
      // The backend command now directly expects the `patch` argument
      const log = await invoke('apply_patch', { patch });
      logToConsole(`✅ Apply successful:\n${log}`, 'success');
      setStatus('applied', 'ok');
      emitAppEvent('apply-successful', { log });
    } catch (e) {
      logToConsole(`❌ Apply failed: ${e}`, 'error');
      setStatus('apply failed', 'err');
      emitAppEvent('apply-failed');
    }
  });

  console.log('[tauri-bridge] Initialized.');
})();