// Xenon Blink host: browser-level callbacks. See xenon_handler.h.
#include "xenon_handler.h"

#include <list>
#include <sstream>
#include <string>

#include "include/wrapper/cef_helpers.h"

#include "include/base/cef_logging.h"

namespace {

// Marks which browser a log line belongs to without ever logging a URL
// or a page title (PRIV-05): browsers are counted, not named.
int BrowserOrdinal(const std::list<CefRefPtr<CefBrowser>>& list,
                   CefRefPtr<CefBrowser> browser) {
  int ordinal = 1;
  for (const auto& entry : list) {
    if (entry->IsSame(browser)) {
      return ordinal;
    }
    ordinal++;
  }
  return 0;
}

}  // namespace

XenonHandler::XenonHandler(bool is_ui_browser) : is_ui_browser_(is_ui_browser) {}

void XenonHandler::OnAfterCreated(CefRefPtr<CefBrowser> browser) {
  CEF_REQUIRE_UI_THREAD();
  browser_list_.push_back(browser);
  LOG(INFO) << "xenon-blink: browser created (ordinal "
            << BrowserOrdinal(browser_list_, browser)
            << ", ui=" << (is_ui_browser_ ? 1 : 0) << ")";
}

bool XenonHandler::DoClose(CefRefPtr<CefBrowser> browser) {
  CEF_REQUIRE_UI_THREAD();
  // Allow the close: nothing blocks it in this skeleton.
  return false;
}

void XenonHandler::OnBeforeClose(CefRefPtr<CefBrowser> browser) {
  CEF_REQUIRE_UI_THREAD();
  browser_list_.remove(browser);
  LOG(INFO) << "xenon-blink: browser closed (ui="
            << (is_ui_browser_ ? 1 : 0) << ")";
}

void XenonHandler::OnTitleChange(CefRefPtr<CefBrowser> browser,
                                 const CefString& title) {
  CEF_REQUIRE_UI_THREAD();
  // PRIV-05: page titles are not logged and, in this skeleton, not yet
  // forwarded anywhere. Tab titles reach the UI through the Bridge in a
  // later sub-step.
}

void XenonHandler::OnLoadError(CefRefPtr<CefBrowser> browser,
                               CefRefPtr<CefFrame> frame,
                               ErrorCode errorCode,
                               const CefString& errorText,
                               const CefString& failedUrl) {
  CEF_REQUIRE_UI_THREAD();
  // PRIV-05: the failed URL is deliberately not logged. Only the
  // failure kind is recorded so failures stay diagnosable.
  LOG(WARNING) << "xenon-blink: load error kind " << errorCode
               << " (ui=" << (is_ui_browser_ ? 1 : 0) << ")";
}

void XenonHandler::CloseAllBrowsers(bool force_close) {
  CEF_REQUIRE_UI_THREAD();
  if (is_closing_) {
    return;  // Close request already in progress.
  }
  is_closing_ = true;

  for (const auto& browser : browser_list_) {
    browser->GetHost()->CloseBrowser(force_close);
  }
}
