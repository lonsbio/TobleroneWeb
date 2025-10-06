// coi-serviceworker.js
// Minimal COOP/COEP shim via Service Worker.
// Adds the headers required for cross-origin isolation on same-origin responses.
//
// Place this file at the site root and register it VERY early in your HTML:
//   <script>
//     if (window.crossOriginIsolated !== true && 'serviceWorker' in navigator && location.protocol === 'https:') {
//       navigator.serviceWorker.register('/coi-serviceworker.js');
//     }
//   </script>

self.addEventListener('install', (event) => {
  // Activate immediately, without waiting for existing SW to stop
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  // Become the controlling SW for all clients ASAP
  event.waitUntil(self.clients.claim());
});

// Helper to clone a response and add/override headers
async function addIsolationHeaders(resp) {
  const newHeaders = new Headers(resp.headers);
  newHeaders.set('Cross-Origin-Opener-Policy', 'same-origin');
  newHeaders.set('Cross-Origin-Embedder-Policy', 'require-corp');
  // Optional but sometimes helpful for static assets you control:
  // newHeaders.set('Cross-Origin-Resource-Policy', 'same-origin');

  // Stream the body through unchanged
  const body = resp.body ? resp.body : null;
  return new Response(body, {
    status: resp.status,
    statusText: resp.statusText,
    headers: newHeaders
  });
}

self.addEventListener('fetch', (event) => {
  const req = event.request;
  const url = new URL(req.url);

  // Only touch same-origin requests. Let cross-origin pass through untouched.
  if (url.origin !== self.location.origin) return;

  // We want to ensure COOP/COEP for all same-origin navigations and assets
  // (HTML, scripts, workers, WASM, etc.), but avoid messing with things like
  // devtools, extension requests, etc.
  event.respondWith((async () => {
    const resp = await fetch(req);

    // Only modify "basic" (same-origin) responses; leave others alone.
    if (resp.type === 'basic') {
      // If headers already present, keep them but ensure the required values.
      // If your server already sets COOP/COEP, this is a no-op.
      return addIsolationHeaders(resp);
    }

    return resp;
  })());
});