/**
 * ui-helpers.js
 * Shared UI utilities. No state, pure functions.
 * No dependencies.
 */
(function() {
  'use strict';

  // Escape HTML for safe rendering
  window.escapeHtml = function(str) {
    return String(str)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  };

  // Update status chip
  window.setStatus = function(text, tone) {
    const el = document.getElementById('status');
    if (!el) return;
    
    el.textContent = text;
    
    const colors = {
      ok: 'var(--ok)',
      warn: 'var(--warn)',
      err: 'var(--err)'
    };
    
    const color = colors[tone] || 'var(--border)';
    el.style.borderColor = color;
    el.style.color = color === 'var(--border)' ? 'var(--text-muted)' : color;
  };

  // Emit log event
  window.logToConsole = function(message, level) {
    window.emitAppEvent('console-log', { message, level: level || 'info' });
  };

  console.log('[ui-helpers] initialized');
})();