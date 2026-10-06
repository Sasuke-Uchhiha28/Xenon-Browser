// Xenon Blink host: browser-level callbacks shared by every browser
// (the UI view and tab views). Based on CEF's sample application
// structure (Rules.md 4.4): windowing and forwarding only — all logic
// lives in core or the UI.
#ifndef XENON_BLINK_APP_XENON_HANDLER_H_
#define XENON_BLINK_APP_XENON_HANDLER_H_

#include <list>

#include "include/cef_client.h"

// Implements CefClient and the handlers the host needs in this skeleton:
// process lifetime tracking and error surfaces. Privileged operations
// (Bridge, scheme serving) live in their own components.
class XenonHandler : public CefClient,
                     public CefDisplayHandler,
                     public CefLifeSpanHandler,
                     public CefLoadHandler {
 public:
  // |is_ui_browser| marks the shared-UI browser so it can never be
  // closed as if it were a tab.
  explicit XenonHandler(bool is_ui_browser);

  // CefClient methods:
  CefRefPtr<CefDisplayHandler> GetDisplayHandler() override { return this; }
  CefRefPtr<CefLifeSpanHandler> GetLifeSpanHandler() override { return this; }
  CefRefPtr<CefLoadHandler> GetLoadHandler() override { return this; }

  // CefDisplayHandler methods:
  void OnTitleChange(CefRefPtr<CefBrowser> browser,
                     const CefString& title) override;

  // CefLifeSpanHandler methods:
  void OnAfterCreated(CefRefPtr<CefBrowser> browser) override;
  bool DoClose(CefRefPtr<CefBrowser> browser) override;
  void OnBeforeClose(CefRefPtr<CefBrowser> browser) override;

  // CefLoadHandler methods:
  void OnLoadError(CefRefPtr<CefBrowser> browser,
                   CefRefPtr<CefFrame> frame,
                   ErrorCode errorCode,
                   const CefString& errorText,
                   const CefString& failedUrl) override;

  // Request that all existing browsers close. |force_close| skips the
  // before-unload confirmation of the tab browsers.
  void CloseAllBrowsers(bool force_close);

  bool IsClosing() const { return is_closing_; }

 private:
  // Live browsers, UI thread only.
  typedef std::list<CefRefPtr<CefBrowser>> BrowserList;
  BrowserList browser_list_;

  const bool is_ui_browser_;
  bool is_closing_ = false;

  IMPLEMENT_REFCOUNTING(XenonHandler);
};

#endif  // XENON_BLINK_APP_XENON_HANDLER_H_
