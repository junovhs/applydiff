/**
 * ui-helpers.js
 * Shared UI utility functions. No state. Interacts with the original DOM.
 */
(function () {
  'use strict';

  const statusChip = document.getElementById('status');
  const copyBriefingBtn = document.getElementById('btn-copy-briefing');
  const patchArea = document.getElementById('patch-area');
  const patchPlaceholder = document.getElementById('patch-placeholder');
  const healthErrorsEl = document.createElement('div'); // Create dynamically
  const healthExchangesEl = document.createElement('div'); // Create dynamically

  // This is a mock function as the health monitor UI was removed from HTML
  window.updateHealthDisplay = function (sessionMetrics) {
    // This function is now a no-op but kept for potential future use.
  };

  window.escapeHtml = function (str) {
    return String(str).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/'/g, '&#039;');
  };

  window.setStatus = function (text, tone) {
    if (!statusChip) return;
    statusChip.textContent = text;
    const colors = { ok: 'var(--ok)', warn: 'var(--warn)', err: 'var(--err)' };
    const color = colors[tone] || 'var(--border)';
    statusChip.style.borderColor = color;
    statusChip.style.color = color === 'var(--border)' ? 'var(--text-muted)' : color;
  };

  // THIS IS THE NEW CRITICAL FUNCTION
  window.enforceThresholds = function () {
    const sessionMetrics = window.AppState.session;
    const errors = sessionMetrics.total_errors;
    const exchanges = sessionMetrics.exchange_count;
    let guidance = '';
    let isBlocked = false;

    if (errors >= 3) {
      guidance = 'High error count. Refresh session.';
      isBlocked = true;
    } else if (exchanges >= 10) {
      guidance = 'Exchange limit reached. Refresh session.';
      isBlocked = true;
    }

    copyBriefingBtn.disabled = isBlocked || !window.AppState.projectRoot;

    // Only block the patch area if a project is loaded AND thresholds are exceeded
    const isDisabled = !window.AppState.projectRoot || isBlocked;
    patchArea.classList.toggle('disabled', isDisabled);

    if (isBlocked) {
      patchPlaceholder.textContent = guidance;
      setStatus(guidance, 'err');
    } else if (window.AppState.projectRoot) {
      patchPlaceholder.textContent = 'Click to Paste Patch or File Request';
    } else {
        patchPlaceholder.textContent = 'Select a project to begin';
    }
  };

  window.logToConsole = function (message, level) {
    window.emitAppEvent('console-log', { message, level: level || 'info' });
  };
  
  // Initial check on load
  if (document.readyState !== 'loading') {
    enforceThresholds();
  } else {
    document.addEventListener('DOMContentLoaded', enforceThresholds);
  }

  console.log('[ui-helpers] Initialized.');
})();