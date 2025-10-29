/**
 * PatchPanel.js
 * Manages the patch input area and associated header buttons.
 */
(function () {
  'use strict';

  const patchArea = document.getElementById('patch-area');
  const placeholder = document.getElementById('patch-placeholder');
  const selectDirBtn = document.getElementById('btn-select-dir');
  const copyBriefingBtn = document.getElementById('btn-copy-briefing');
  const refreshBtn = document.getElementById('btn-refresh-session');

  let editorEl = null;
  let typingTimer = null;

  function init() {
    // Wire up header buttons
    selectDirBtn.addEventListener('click', () => emitAppEvent('select-project-requested'));
    copyBriefingBtn.addEventListener('click', () => emitAppEvent('copy-briefing-requested'));
    if (refreshBtn) {
      refreshBtn.addEventListener('click', () => emitAppEvent('refresh-session-requested'));
    }

    patchArea.addEventListener('click', onPatchAreaClick);

    // App event listeners
    onAppEvent('session-loaded', (e) => {
      // THIS IS THE RESTORED GUIDANCE
      logToConsole(`
      --------------------------------------------------
      ✅ Project Loaded. To test the workflow:

      1. Create a file named 'test.txt' in '${e.detail.path}'.
      2. Put the word 'hello' inside it.
      3. Copy the patch below and paste it into the patch panel.

      >>> file: test.txt
      --- from
      hello
      --- to
      goodbye
      <<<
      --------------------------------------------------
      `, 'success');

      placeholder.textContent = 'Click to Paste Patch';
      patchArea.classList.remove('disabled');
      copyBriefingBtn.disabled = false;
      if (refreshBtn) refreshBtn.disabled = false;
      
      setStatus('ready');
      emitAppEvent('session-state-sync-requested');
    });
    
    onAppEvent('session-load-failed', () => {
      placeholder.textContent = 'Select a project to begin';
      patchArea.classList.add('disabled');
      copyBriefingBtn.disabled = true;
      if (refreshBtn) refreshBtn.disabled = true;
    });

    onAppEvent('apply-successful', () => {
      if (editorEl) {
        editorEl.value = '';
        window.AppState.currentPatch = '';
        placeholder.style.display = 'block';
      }
      setStatus('idle');
    });
  }

  async function onPatchAreaClick() {
    if (patchArea.classList.contains('disabled')) return;
    ensureEditorExists();
    try {
      const textFromClipboard = await window.__TAURI__.clipboardManager.readText();
      if (textFromClipboard && textFromClipboard.trim()) {
        logToConsole(`📋 Pasted ${textFromClipboard.length} chars from clipboard.`);
        editorEl.value = textFromClipboard;
        window.AppState.currentPatch = textFromClipboard;
        placeholder.style.display = 'none';
        emitAppEvent('preview-requested', { patch: textFromClipboard });
        editorEl.focus();
      } else {
        logToConsole('⌨️ Clipboard empty. Enter patch manually or copy the example from the console.', 'warn');
        editorEl.focus();
      }
    } catch (e) {
      logToConsole(`❌ Clipboard read failed: ${e}`, 'error');
      editorEl.focus();
    }
  }

  function ensureEditorExists() {
    if (editorEl) return;
    editorEl = document.createElement('textarea');
    editorEl.className = 'patch-editor';
    patchArea.appendChild(editorEl);
    editorEl.addEventListener('input', () => {
      setStatus('typing…', 'warn');
      clearTimeout(typingTimer);
      typingTimer = setTimeout(() => {
        const patch = editorEl.value.trim();
        if (patch) {
          window.AppState.currentPatch = patch;
          emitAppEvent('preview-requested', { patch });
        }
      }, 1500);
    });
  }

  // Auto-init
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();