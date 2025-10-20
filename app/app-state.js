/**
 * app-state.js
 * Global application state and event helpers.
 * No dependencies.
 */
(function() {
  'use strict';

  // Initialize global state
  window.AppState = {
    selectedDir: '',
    currentPatch: '',
    versions: [],
    currentVersion: -1,
    consoleVisible: false,
    previewInFlight: false
  };

  // Event helper: emit custom event
  window.emitAppEvent = function(name, detail) {
    window.dispatchEvent(new CustomEvent('app:' + name, { detail: detail || {} }));
  };

  // Event helper: listen to app event
  window.onAppEvent = function(name, handler) {
    window.addEventListener('app:' + name, handler);
  };

  console.log('[app-state] initialized');
})();