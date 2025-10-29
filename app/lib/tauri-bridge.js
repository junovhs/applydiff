/**
 * tauri-bridge.js
 * All Tauri IPC calls. Listens for UI events, calls the backend, and emits results.
 */
(function () {
  'use strict';

  const { invoke } = window.__TAURI__.core;

  onAppEvent('select-project-requested', async () => {
    try {
      logToConsole('📁 Requesting project directory...');
      const projectRoot = await invoke('load_session');
      if (projectRoot) {
        window.AppState.projectRoot = projectRoot;
        emitAppEvent('session-loaded', { path: projectRoot });
      }
    } catch (e) { logToConsole(`❌ Project selection failed: ${e}`, 'error'); emitAppEvent('session-load-failed'); }
  });

  onAppEvent('copy-briefing-requested', async () => {
    if (!window.AppState.projectRoot) return;
    try {
      const briefing = await invoke('get_session_briefing');
      await window.__TAURI__.clipboardManager.writeText(briefing);
      logToConsole('📋 AI briefing copied to clipboard.');
      emitAppEvent('session-state-sync-requested');
    } catch (e) { logToConsole(`❌ Failed to get AI briefing: ${e}`, 'error'); }
  });
  
  onAppEvent('refresh-session-requested', async () => {
    if (!window.AppState.projectRoot) return;
    try {
      await invoke('refresh_session');
      logToConsole('🔄️ Session counters have been refreshed.');
      emitAppEvent('session-state-sync-requested');
    } catch (e) { logToConsole(`❌ Failed to refresh session: ${e}`, 'error'); }
  });

  onAppEvent('preview-requested', async (e) => {
    const { patch } = e.detail;
    if (!window.AppState.projectRoot || !patch || window.AppState.ui.isPreviewInFlight) return;

    window.AppState.ui.isPreviewInFlight = true;
    setStatus('previewing…', 'warn');
    try {
      const result = await invoke('preview_patch', { patch });
      const hasDiff = result && result.diff && result.diff.trim();
      const hasError = result && /❌/.test(result.log || '');
      
      // A failed preview now updates session state, so we must sync.
      if (hasError) {
          emitAppEvent('session-state-sync-requested');
      }

      emitAppEvent('preview-ready', { diff: result.diff || '', log: result.log || '', hasDiff, hasError });

      if (!hasDiff && !hasError) setStatus('ready');
      else if (hasError) setStatus('preview error', 'err');
      else setStatus('ready to apply', 'ok');

    } catch (e) {
      logToConsole(`❌ Preview failed: ${e}`, 'error');
      setStatus('preview error', 'err');
      emitAppEvent('preview-ready', { diff: '', log: e, hasDiff: false, hasError: true });
    } finally { window.AppState.ui.isPreviewInFlight = false; }
  });

  onAppEvent('apply-requested', async (e) => {
    const { patch } = e.detail;
    if (!window.AppState.projectRoot || !patch) return;

    setStatus('applying…', 'warn');
    try {
      const log = await invoke('apply_patch', { patch });
      logToConsole(`✅ Apply successful:\n${log}`);
      setStatus('applied', 'ok');
      emitAppEvent('apply-successful', { log });
      emitAppEvent('session-state-sync-requested');
    } catch (e) {
      logToConsole(`❌ Apply failed: ${e}`, 'error');
      setStatus('apply failed', 'err');
      emitAppEvent('apply-failed');
      emitAppEvent('session-state-sync-requested');
    }
  });
  
  onAppEvent('session-state-sync-requested', async () => {
    try {
      const sessionState = await invoke('get_session_state');
      window.AppState.session = sessionState;
      emitAppEvent('session-state-updated', { session: sessionState });
    } catch (e) { logToConsole(`⚠️ Could not sync session state: ${e}`, 'warn'); }
  });

  onAppEvent('session-state-updated', (e) => {
    window.updateHealthDisplay(e.detail.session);
    // THIS IS THE CRITICAL FIX for threshold enforcement
    window.enforceThresholds(e.detail.session);
    logToConsole('UI state synced with backend.');
  });

  console.log('[tauri-bridge] Initialized.');
})();
