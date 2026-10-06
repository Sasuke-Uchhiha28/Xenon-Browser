// Xenon Blink host: application-level callbacks. Based on CEF's sample
// application structure (Rules.md 4.4).
//
// Window layout (Architecture.md 7, Blink column): the shared UI is ONE
// Alloy-style BrowserView filling the whole window; the active tab is an
// Alloy-style BrowserView added as a window overlay positioned over the
// UI's content area (below the 72px chrome, right of the 56px sidebar
// rail). The overlay receives input inside its bounds; the UI keeps
// input everywhere else. Popovers will later use the same overlay
// mechanism (spike S2).
#ifndef XENON_BLINK_APP_XENON_APP_H_
#define XENON_BLINK_APP_XENON_APP_H_

#include "include/cef_app.h"

class XenonApp : public CefApp, public CefBrowserProcessHandler {
 public:
  XenonApp();

  // CefApp methods:
  void OnRegisterCustomSchemes(CefRawPtr<CefSchemeRegistrar> registrar) override;
  CefRefPtr<CefBrowserProcessHandler> GetBrowserProcessHandler() override {
    return this;
  }

  // CefBrowserProcessHandler methods:
  void OnContextInitialized() override;

 private:
  IMPLEMENT_REFCOUNTING(XenonApp);
  DISALLOW_COPY_AND_ASSIGN(XenonApp);
};

#endif  // XENON_BLINK_APP_XENON_APP_H_
