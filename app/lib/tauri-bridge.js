/**
 * tauri-bridge.js
 * All Tauri IPC calls. Listens for UI events, calls the backend, and emits results.
 */
(function () {
  'use strict';

  const { invoke } = window.__TAURI__.core;

  function syncSessionState(sessionState) {
    if (!sessionState) return;
    window.AppState.session.exchange_count = sessionState.exchange_count;
    window.AppState.session.total_errors = sessionState.total_errors;
    emitAppEvent('session-state-updated');
  }

  // User wants to select a project directory and initialize a session
  onAppEvent('select-project-requested', async () => {
    try {
      logToConsole('📁 Initializing session...');
      const sessionState = await invoke('init_session');
      if (sessionState && sessionState.project_root) {
        window.AppState.projectRoot = sessionState.project_root;
        syncSessionState(sessionState);
        emitAppEvent('project-loaded', { path: sessionState.project_root });
      }
    } catch (e) {
      logToConsole(`❌ Session initialization failed: ${e}`, 'error');
      emitAppEvent('project-load-failed');
    }
  });

  // User wants the dynamic session briefing
  onAppEvent('copy-briefing-requested', async () => {
    try {
      const briefing = await invoke('get_session_briefing');
      await window.__TAURI__.clipboardManager.writeText(briefing);
      logToConsole('📋 Session briefing copied to clipboard.');
    } catch (e) {
      logToConsole(`❌ Failed to copy briefing: ${e}`, 'error');
    }
  });

  // User wants to refresh the session
  onAppEvent('refresh-session-requested', async () => {
    try {
      logToConsole('🔄 Refreshing session...');
      const sessionState = await invoke('refresh_session');
      syncSessionState(sessionState);
      logToConsole('✅ Session refreshed successfully.');
      setStatus('refreshed', 'ok');
    } catch (e) {
      logToConsole(`❌ Failed to refresh session: ${e}`, 'error');
      setStatus('refresh failed', 'err');
    }
  });

  // A file request has been pasted
  onAppEvent('resolve-file-request-requested', async (e) => {
    const { requestYaml } = e.detail;
    if (!window.AppState.projectRoot) return;
    setStatus('resolving…', 'warn');
    try {
      const markdown = await invoke('resolve_file_request', { requestYaml });
      await window.__TAURI__.clipboardManager.writeText(markdown);
      logToConsole('✅ File request resolved. Markdown is now on your clipboard.', 'success');
      setStatus('resolved', 'ok');
    } catch (e) {
      logToConsole(`❌ File request failed: ${e}`, 'error');
      setStatus('resolve failed', 'err');
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

      emitAppEvent('preview-ready', { diff: result.diff, log: result.log, hasDiff, hasError });

      if (!hasDiff && !hasError) { setStatus('ready'); }
      else if (hasError) { setStatus('preview error', 'err'); }
      else { setStatus('ready to apply', 'ok'); }

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
      const result = await invoke('apply_patch', { patch });
      logToConsole(`✅ Apply successful:\n${result.output}`, 'success');
      syncSessionState(result.session_state);
      setStatus('applied', 'ok');
      emitAppEvent('apply-successful');
    } catch (e) {
      logToConsole(`❌ Apply failed: ${e}`, 'error');
      setStatus('apply failed', 'err');
      emitAppEvent('apply-failed');
    }
  });

  console.log('[tauri-bridge] Initialized.');
})();