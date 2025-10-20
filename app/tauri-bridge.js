/**
 * tauri-bridge.js
 * All Tauri IPC calls. Listens for requests, emits results.
 * No direct DOM manipulation.
 */
(function() {
  'use strict';

  // Tauri invoke wrapper
  function invoke(...args) {
    if (!window.__TAURI__ || !window.__TAURI__.core) {
      throw new Error('Tauri core not available');
    }
    return window.__TAURI__.core.invoke(...args);
  }

  // Pick folder
  window.onAppEvent('pick-folder-requested', async () => {
    try {
      const dir = await invoke('pick_folder');
      if (typeof dir === 'string' && dir.length) {
        window.AppState.selectedDir = dir;
        window.emitAppEvent('directory-selected', { path: dir });
        logToConsole('📁 Selected: ' + dir);
      }
    } catch (e) {
      logToConsole('❌ Select directory failed: ' + e, 'error');
    }
  });

  // Copy AI prompt
  window.onAppEvent('copy-prompt-requested', async () => {
    try {
      const prompt = await invoke('get_ai_prompt');
      await navigator.clipboard.writeText(String(prompt || ''));
      logToConsole('📋 AI prompt copied.');
    } catch (e) {
      logToConsole('❌ Copy AI prompt failed: ' + e, 'error');
    }
  });

  // Preview patch
  window.onAppEvent('preview-requested', async (e) => {
    const { patch } = e.detail;
    const dir = window.AppState.selectedDir;
    
    if (!dir || !patch) return;
    if (window.AppState.previewInFlight) return;
    
    window.AppState.previewInFlight = true;
    window.setStatus('previewing…', 'warn');
    
    try {
      const res = await invoke('preview_patch', { target: dir, patch });
      
      if (res && res.log) {
        const tail = res.log.split('\n').slice(-40).join('\n');
        logToConsole('👁 Preview:\n' + tail);
      }
      
      const hasDiff = !!(res?.diff && res.diff.trim());
      const hasError = /❌/.test(res?.log || '');
      
      window.emitAppEvent('preview-ready', {
        diff: res?.diff || '',
        hasError,
        hasDiff
      });
      
      if (!hasDiff) {
        window.setStatus('idle');
      } else if (hasError) {
        window.setStatus('partial: some blocks failed', 'warn');
      } else {
        window.setStatus('ready to apply', 'ok');
      }
    } catch (e) {
      logToConsole('❌ Preview error: ' + e, 'error');
      window.setStatus('error', 'err');
      window.emitAppEvent('preview-ready', { diff: '', hasError: true, hasDiff: false });
    } finally {
      window.AppState.previewInFlight = false;
    }
  });

  // Apply patch
  window.onAppEvent('apply-requested', async (e) => {
    const { patch, diff } = e.detail;
    const dir = window.AppState.selectedDir;
    
    if (!dir || !patch) return;
    
    window.setStatus('applying…', 'warn');
    
    try {
      const out = await invoke('apply_patch', { target: dir, patch });
      logToConsole('✅ Apply:\n' + out);
      window.setStatus('applied', 'ok');
      
      window.emitAppEvent('apply-complete', { patch, diff, log: out });
    } catch (e) {
      logToConsole('❌ Apply failed: ' + e, 'error');
      window.setStatus('apply failed', 'err');
    }
  });

  // Run self-test
  window.onAppEvent('self-test-requested', async () => {
    try {
      logToConsole('🧪 Running tests…');
      const out = await invoke('run_self_test');
      logToConsole(out);
    } catch (e) {
      logToConsole('❌ Self-test error: ' + e, 'error');
    }
  });

  console.log('[tauri-bridge] initialized');
})();