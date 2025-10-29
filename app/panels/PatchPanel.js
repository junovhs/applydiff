/**
 * PatchPanel.js
 * Manages the patch input area and associated header buttons.
 */
(function () {
  'use strict';

  // Get DOM elements
  const patchArea = document.getElementById('patch-area');
  const placeholder = document.getElementById('patch-placeholder');
  const selectDirBtn = document.getElementById('btn-select-dir');
  const copyBriefingBtn = document.getElementById('btn-copy-briefing');
  const refreshBtn = document.getElementById('btn-refresh-session');

  // Module-level state
  let editorEl = null;
  let typingTimer = null;

  // Function to create the editor if it doesn't exist
  function ensureEditorExists() {
    if (editorEl) {
      return;
    }

    editorEl = document.createElement('textarea');
    editorEl.className = 'patch-editor';
    patchArea.appendChild(editorEl);

    editorEl.addEventListener('input', () => {
      window.setStatus('typing…', 'warn');
      clearTimeout(typingTimer);
      typingTimer = setTimeout(() => {
        const patch = editorEl.value.trim();
        if (patch) {
          window.AppState.currentPatch = patch;
          window.emitAppEvent('preview-requested', { patch });
        }
      }, 1500);
    });
  }

  // Handler for clicking the patch area to paste
  async function onPatchAreaClick() {
    if (patchArea.classList.contains('disabled')) {
      return;
    }

    ensureEditorExists();

    try {
      const textFromClipboard = await window.__TAURI__.clipboardManager.readText();

      if (textFromClipboard && textFromClipboard.trim()) {
        logToConsole(`📋 Pasted ${textFromClipboard.length} chars from clipboard.`);
        editorEl.value = textFromClipboard;
        window.AppState.currentPatch = textFromClipboard;
        placeholder.style.display = 'none';
        window.emitAppEvent('preview-requested', { patch: textFromClipboard });
        editorEl.focus();
      } else {
        logToConsole('⌨️ Clipboard empty. Enter patch manually.');
        editorEl.focus();
      }
    } catch (error) {
      logToConsole(`❌ Clipboard read failed: ${error}`, 'error');
      // Still focus the editor for manual input
      editorEl.focus();
    }
  }

  // Main initialization function
  function init() {
    // Wire up header buttons
    selectDirBtn.addEventListener('click', () => {
      window.emitAppEvent('select-project-requested');
    });

    copyBriefingBtn.addEventListener('click', () => {
      window.emitAppEvent('copy-briefing-requested');
    });

    if (refreshBtn) {
      refreshBtn.addEventListener('click', () => {
        window.emitAppEvent('refresh-session-requested');
      });
    }

    // Main event listener
    patchArea.addEventListener('click', onPatchAreaClick);

    // App event listeners
    window.onAppEvent('session-loaded', (e) => {
      placeholder.textContent = 'Click to Paste Patch';
      patchArea.classList.remove('disabled');
      copyBriefingBtn.disabled = false;
      if (refreshBtn) refreshBtn.disabled = false;
      window.logToConsole(`Project loaded: ${e.detail.path}`);
      window.setStatus('ready');
      window.emitAppEvent('session-state-sync-requested');
    });

    window.onAppEvent('session-load-failed', () => {
      placeholder.textContent = 'Select a project to begin';
      patchArea.classList.add('disabled');
      copyBriefingBtn.disabled = true;
      if (refreshBtn) refreshBtn.disabled = true;
    });

    window.onAppEvent('apply-successful', () => {
      if (editorEl) {
        editorEl.value = '';
        window.AppState.currentPatch = '';
        placeholder.style.display = 'block';
      }
      window.setStatus('idle');
    });
  }

  // Run init
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();