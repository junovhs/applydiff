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
    selectDirBtn.addEventListener('click', () => emitAppEvent('select-project-requested'));
    copyBriefingBtn.addEventListener('click', () => emitAppEvent('copy-briefing-requested'));
    refreshBtn.addEventListener('click', () => emitAppEvent('refresh-session-requested'));

    patchArea.addEventListener('click', onPatchAreaClick);

    onAppEvent('project-loaded', (e) => {
      logToConsole(`✅ Project loaded at: ${e.detail.path}\nSession initialized.`, 'success');
      refreshBtn.disabled = false;
      enforceThresholds();
    });
    
    onAppEvent('project-load-failed', () => {
      enforceThresholds();
    });

    onAppEvent('session-state-updated', () => {
      enforceThresholds();
    });

    onAppEvent('apply-successful', () => {
      if (editorEl) {
        editorEl.value = '';
        window.AppState.currentPatch = '';
        placeholder.style.display = 'block';
        editorEl.style.display = 'none';
      }
      setStatus('idle');
    });
  }

  async function onPatchAreaClick() {
    if (patchArea.classList.contains('disabled')) return;
    
    try {
      const clipboardText = await window.__TAURI__.clipboardManager.readText();
      if (!clipboardText || !clipboardText.trim()) {
        logToConsole('Clipboard is empty. Paste a patch or REQUEST_FILE block.', 'warn');
        ensureEditorExists(); // Still show editor for manual input
        editorEl.focus();
        return;
      }
      
      const trimmedText = clipboardText.trim();
      if (trimmedText.toUpperCase().startsWith('REQUEST_FILE:')) {
        logToConsole('👁️ REQUEST_FILE protocol detected. Resolving...', 'info');
        // The request string needs to be the text *after* the initial keyword
        const requestYaml = trimmedText.substring(trimmedText.indexOf(':') + 1).trim();
        emitAppEvent('resolve-file-request-requested', { requestYaml });
      } else {
        logToConsole(`📋 Pasted ${clipboardText.length} chars from clipboard.`);
        ensureEditorExists();
        editorEl.value = clipboardText;
        window.AppState.currentPatch = clipboardText;
        placeholder.style.display = 'none';
        editorEl.style.display = 'block';
        emitAppEvent('preview-requested', { patch: clipboardText });
        editorEl.focus();
      }
    } catch (e) {
      logToConsole(`❌ Clipboard read failed: ${e}`, 'error');
      ensureEditorExists();
      editorEl.focus();
    }
  }

  function ensureEditorExists() {
    if (editorEl) return;
    editorEl = document.createElement('textarea');
    editorEl.className = 'patch-editor';
    patchArea.appendChild(editorEl);
    
    editorEl.addEventListener('focus', () => {
        placeholder.style.display = 'none';
    });
    
    editorEl.addEventListener('blur', () => {
        if (!editorEl.value.trim()) {
            placeholder.style.display = 'block';
        }
    });

    editorEl.addEventListener('input', () => {
      setStatus('typing…', 'warn');
      const patch = editorEl.value;
      placeholder.style.display = patch ? 'none' : 'block';
      
      clearTimeout(typingTimer);
      typingTimer = setTimeout(() => {
        const trimmedPatch = patch.trim();
        if (trimmedPatch) {
          window.AppState.currentPatch = trimmedPatch;
          emitAppEvent('preview-requested', { patch: trimmedPatch });
        }
      }, 1500);
    });
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();