// Serves the shared UI (desktop/ui build output) over the xenon://ui
// scheme so the privileged UI document loads with module scripts, fonts
// and fetch working — no network listener, no file:// CORS problems
// (Architecture.md 4.2, 12).
#ifndef XENON_BLINK_APP_UI_SCHEME_HANDLER_H_
#define XENON_BLINK_APP_UI_SCHEME_HANDLER_H_

#include "include/cef_scheme.h"

// Registers the xenon scheme in every process. Called from CefApp
// OnRegisterCustomSchemes; must match in the browser and render
// processes or module scripts will fail CORS.
void RegisterXenonSchemes(CefRawPtr<CefSchemeRegistrar> registrar);

// Registers the factory that serves files for xenon://ui/* from
// |ui_root| (the desktop/ui build output directory). Call on the UI
// thread after CEF context initialization.
void RegisterUiSchemeHandler(const std::string& ui_root);

#endif  // XENON_BLINK_APP_UI_SCHEME_HANDLER_H_
