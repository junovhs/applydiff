/**
 * ui-helpers.js
 * Shared UI utility functions. No state. Interacts with the original DOM.
 */
(function () {
  'use strict';

  const statusChip = document.getElementById('status');
  const healthErrorsEl = document.getElementById('health-errors');
  const healthExchangesEl = document.getElementById('health-exchanges');
  const copyBriefingBtn = document.getElementById('btn-copy-briefing');
  const patchArea = document.getElementById('patch-area');
  const patchPlaceholder = document.getElementById('patch-placeholder');

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

  window.updateHealthDisplay = function (sessionMetrics) {
    if (healthErrorsEl) {
      healthErrorsEl.textContent = `Errors: ${sessionMetrics.total_errors}/3`;
      healthErrorsEl.style.color = 'var(--text-muted)';
      if (sessionMetrics.total_errors >= 2) healthErrorsEl.style.color = 'var(--warn)';
      if (sessionMetrics.total_errors >= 3) healthErrorsEl.style.color = 'var(--err)';
    }
    if (healthExchangesEl) {
      healthExchangesEl.textContent = `Exchanges: ${sessionMetrics.exchange_count}/10`;
      healthExchangesEl.style.color = 'var(--text-muted)';
      if (sessionMetrics.exchange_count >= 10) healthExchangesEl.style.color = 'var(--warn)';
    }
  };

  // THIS IS THE NEW CRITICAL FUNCTION
  window.enforceThresholds = function (sessionMetrics) {
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

    copyBriefingBtn.disabled = isBlocked;
    patchArea.classList.toggle('disabled', isBlocked);

    if (isBlocked) {
      patchPlaceholder.textContent = guidance;
      setStatus(guidance, 'err');
    } else if (window.AppState.projectRoot) {
      patchPlaceholder.textContent = 'Click to Paste Patch';
    }
  };

  window.logToConsole = function (message, level) {
    window.emitAppEvent('console-log', { message, level: level || 'info' });
  };

  console.log('[ui-helpers] Initialized.');
})();