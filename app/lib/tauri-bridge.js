/**
 * tauri-bridge.js
 * All Tauri IPC calls. Listens for UI events, calls the backend, and emits results.
 */
(function () {
  'use strict';

  const { invoke } = window.__TAURI__.core;

  // --- Listen for UI requests and call the backend ---

  // User wants to select a project directory
  onAppEvent('select-project-requested', async () => {
    try {
      logToConsole('📁 Requesting project directory...');
      const projectRoot = await invoke('load_session');
      if (projectRoot) {
        window.AppState.projectRoot = projectRoot;
        emitAppEvent('session-loaded', { path: projectRoot });
      }
    } catch (e) {
      logToConsole(`❌ Project selection failed: ${e}`, 'error');
      emitAppEvent('session-load-failed');
    }
  });

  // User wants the AI briefing
  onAppEvent('copy-briefing-requested', async () => {
    if (!window.AppState.projectRoot) return;
    try {
      const briefing = await invoke('get_session_briefing');
      await window.__TAURI__.clipboardManager.writeText(briefing);
      logToConsole('📋 AI briefing copied to clipboard.');
      // The backend increments the session count, so we need to sync state
      emitAppEvent('session-state-sync-requested');
    } catch (e) {
      logToConsole(`❌ Failed to get AI briefing: ${e}`, 'error');
    }
  });
  
  // User wants to reset the session counters
  onAppEvent('refresh-session-requested', async () => {
    if (!window.AppState.projectRoot) return;
    try {
        await invoke('refresh_session');
        logToConsole('🔄️ Session counters have been refreshed.');
        emitAppEvent('session-state-sync-requested');
    } catch (e) {
        logToConsole(`❌ Failed to refresh session: ${e}`, 'error');
    }
  });

  // A patch has been entered and needs a preview
  onAppEvent('preview-requested', async (e) => {
    const { patch } = e.detail;
    if (!window.AppState.projectRoot || !patch || window.AppState.ui.isPreviewInFlight) return;

    window.AppState.ui.isPreviewInFlight = true;
    setStatus('previewing…', 'warn');
    try {
      const result = await invoke('preview_patch', { patch });
      const hasDiff = result && result.diff && result.diff.trim();
      const hasError = result && /❌/.test(result.log || '');

      emitAppEvent('preview-ready', {
        diff: result.diff || '',
        log: result.log || '',
        hasDiff,
        hasError,
      });

      if (!hasDiff && !hasError) setStatus('ready');
      else if (hasError) setStatus('partial match', 'warn');
      else setStatus('ready to apply', 'ok');

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
    if (!window.AppState.projectRoot || !patch) return;

    setStatus('applying…', 'warn');
    try {
      const log = await invoke('apply_patch', { patch });
      logToConsole(`✅ Apply successful:\n${log}`);
      setStatus('applied', 'ok');
      emitAppEvent('apply-successful', { log });
      // Sync state to reflect new error/success counts
      emitAppEvent('session-state-sync-requested');
    } catch (e) {
      logToConsole(`❌ Apply failed: ${e}`, 'error');
      setStatus('apply failed', 'err');
      emitAppEvent('apply-failed');
      // Sync state to reflect the new error count
      emitAppEvent('session-state-sync-requested');
    }
  });
  
  // A component needs to get the latest session state from the backend
  onAppEvent('session-state-sync-requested', async () => {
    try {
        const sessionState = await invoke('get_session_state');
        window.AppState.session = sessionState;
        emitAppEvent('session-state-updated', { session: sessionState });
    } catch (e) {
        logToConsole(`⚠️ Could not sync session state: ${e}`, 'warn');
    }
  });


  console.log('[tauri-bridge] Initialized.');
})();