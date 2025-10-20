/**
 * patch-panel.js
 * Manages patch input area, paste/type handling.
 */
(function() {
  'use strict';

  let patchArea, placeholder, editorEl;
  let typingTimer = null;

  function init() {
    patchArea = document.getElementById('patch-area');
    placeholder = document.getElementById('patch-placeholder');
    
    // Enable patch area when directory selected
    window.onAppEvent('directory-selected', () => {
      patchArea.classList.remove('disabled');
      window.setStatus('ready');
    });
    
    // Clear editor after apply
    window.onAppEvent('apply-complete', () => {
      clearEditor();
    });
    
    // Repopulate editor when version selected
    window.onAppEvent('version-selected', (e) => {
      const { patch } = e.detail;
      if (patch !== undefined) {
        ensureEditor();
        editorEl.value = patch;
        placeholder.style.display = patch ? 'none' : 'block';
      }
    });
    
    // Click to paste
    patchArea.addEventListener('click', handlePatchAreaClick);
    
    // Header button handlers
    document.getElementById('btn-select-dir').addEventListener('click', () => {
      window.emitAppEvent('pick-folder-requested');
    });
    
    document.getElementById('btn-copy-prompt').addEventListener('click', () => {
      window.emitAppEvent('copy-prompt-requested');
    });
  }

  async function handlePatchAreaClick() {
    if (!window.AppState.selectedDir) {
      logToConsole('ℹ️ Select a directory first.');
      return;
    }
    
    // Try to read clipboard
    let text = await tryReadClipboard();
    
    if (text && text.trim()) {
      logToConsole('📋 Pasted ' + text.length + ' chars');
      ensureEditor();
      editorEl.value = text;
      placeholder.style.display = 'none';
      window.setStatus('previewing…', 'warn');
      window.emitAppEvent('preview-requested', { patch: text });
      editorEl.focus();
      return;
    }
    
    // Fallback: typing mode
    logToConsole('⌨️ Typing mode. Auto-preview after 3s idle.');
    ensureEditor();
    placeholder.style.display = 'none';
    editorEl.focus();
  }

  async function tryReadClipboard() {
    // Try Tauri clipboard first
    try {
      if (window.__TAURI__?.clipboardManager?.readText) {
        const text = await window.__TAURI__.clipboardManager.readText();
        if (text && text.trim()) return text;
      }
    } catch (e) {
      // Silent fallback
    }
    
    // Try browser clipboard
    try {
      const text = await navigator.clipboard.readText();
      if (text && text.trim()) return text;
    } catch (e) {
      // Silent fallback
    }
    
    return null;
  }

  function ensureEditor() {
    if (editorEl) return;
    
    editorEl = document.createElement('textarea');
    editorEl.id = 'patch-editor';
    editorEl.className = 'patch-editor';
    patchArea.appendChild(editorEl);
    
    editorEl.addEventListener('input', () => {
      if (!window.AppState.selectedDir) return;
      
      window.setStatus('typing…', 'warn');
      
      if (typingTimer) clearTimeout(typingTimer);
      typingTimer = setTimeout(() => {
        const patch = editorEl.value;
        if (patch.trim()) {
          window.emitAppEvent('preview-requested', { patch });
        }
      }, 3000);
    });
  }

  function clearEditor() {
    if (editorEl) {
      editorEl.value = '';
      placeholder.style.display = 'block';
    }
    window.setStatus('idle');
  }

  // Auto-init
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }

  console.log('[patch-panel] initialized');
})();