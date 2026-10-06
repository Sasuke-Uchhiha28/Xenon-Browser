// Xenon Blink host: application-level callbacks. See xenon_app.h.
#include "xenon_app.h"

#include <string>

#include "ui_scheme_handler.h"
#include "xenon_handler.h"
#include "include/cef_browser.h"
#include "include/cef_command_line.h"
#include "include/views/cef_browser_view.h"
#include "include/views/cef_fill_layout.h"
#include "include/views/cef_window.h"
#include "include/base/cef_bind.h"
#include "include/cef_task.h"
#include "include/wrapper/cef_helpers.h"

namespace {

// Chrome geometry from Design.md 4.2: two chrome rows totalling 72px and
// a 56px sidebar rail. The tab overlay covers the remaining content
// area. Sub-step 3 moves these numbers into a live layout negotiation
// with the UI (Bridge v0 "window.layout").
constexpr int kChromeHeight = 72;
constexpr int kRailWidth = 56;

CefRect ContentRectFor(const CefRect& window_bounds) {
  return CefRect(kRailWidth, kChromeHeight,
                 window_bounds.width - kRailWidth,
                 window_bounds.height - kChromeHeight);
}

// Position and show the overlay over the content area, mirroring the
// reference implementation (tests/cefclient/browser/
// views_overlay_browser.cc): SetSize, SetBounds, then SetVisible.
void UpdateOverlay(CefRefPtr<CefWindow> window,
                   CefRefPtr<CefOverlayController> overlay) {
  if (!overlay) {
    return;
  }
  const CefRect& content = ContentRectFor(window->GetBounds());
  if (content.width > 0 && content.height > 0) {
    overlay->SetSize(CefSize(content.width, content.height));
    overlay->SetBounds(content);
    overlay->SetVisible(true);
  } else {
    overlay->SetVisible(false);
  }
}

// Window delegate: hosts the shared-UI browser view as the window
// content and the active tab as an overlay over the content area.
class XenonWindowDelegate : public CefWindowDelegate {
 public:
  XenonWindowDelegate(CefRefPtr<CefBrowserView> ui_view,
                      CefRefPtr<CefBrowserView> tab_view,
                      const std::string& url)
      : ui_view_(ui_view), tab_view_(tab_view), url_(url) {}

  XenonWindowDelegate(const XenonWindowDelegate&) = delete;
  XenonWindowDelegate& operator=(const XenonWindowDelegate&) = delete;

  void OnWindowCreated(CefRefPtr<CefWindow> window) override {
    window->SetToFillLayout();
    window->AddChildView(ui_view_);
    window->Show();
    if (tab_view_) {
      overlay_ = window->AddOverlayView(tab_view_, CEF_DOCKING_MODE_CUSTOM,
                                        true);
      UpdateOverlay(window, overlay_);
      CefRefPtr<CefBrowser> tab = tab_view_->GetBrowser();
      if (tab) {
        tab->GetMainFrame()->LoadURL(url_);
      }
    }
  }

  void OnWindowBoundsChanged(CefRefPtr<CefWindow> window,
                             const CefRect& new_bounds) override {
    UpdateOverlay(window, overlay_);
  }

  void OnWindowDestroyed(CefRefPtr<CefWindow> window) override {
    ui_view_ = nullptr;
    tab_view_ = nullptr;
    overlay_ = nullptr;
  }

  bool CanClose(CefRefPtr<CefWindow> window) override {
    // Close all tab and UI browsers; CEF confirms before-unload itself.
    CefRefPtr<CefBrowser> tab = tab_view_ ? tab_view_->GetBrowser() : nullptr;
    if (tab && !tab->GetHost()->TryCloseBrowser()) {
      return false;
    }
    CefRefPtr<CefBrowser> ui = ui_view_ ? ui_view_->GetBrowser() : nullptr;
    if (ui && !ui->GetHost()->TryCloseBrowser()) {
      return false;
    }
    return true;
  }

  CefSize GetPreferredSize(CefRefPtr<CefView> view) override {
    return CefSize(1200, 800);
  }

  // The shared UI requires Alloy-style views (Architecture.md 7).
  cef_runtime_style_t GetWindowRuntimeStyle() override {
    return CEF_RUNTIME_STYLE_ALLOY;
  }

 private:
  CefRefPtr<CefWindow> window_;
  CefRefPtr<CefBrowserView> ui_view_;
  CefRefPtr<CefBrowserView> tab_view_;
  CefRefPtr<CefOverlayController> overlay_;
  std::string url_;

  IMPLEMENT_REFCOUNTING(XenonWindowDelegate);
};

class XenonBrowserViewDelegate : public CefBrowserViewDelegate {
 public:
  XenonBrowserViewDelegate() = default;

  XenonBrowserViewDelegate(const XenonBrowserViewDelegate&) = delete;
  XenonBrowserViewDelegate& operator=(const XenonBrowserViewDelegate&) =
      delete;

  // Popups (target=_blank, DevTools) open in their own top-level window.
  bool OnPopupBrowserViewCreated(CefRefPtr<CefBrowserView> browser_view,
                                 CefRefPtr<CefBrowserView> popup_browser_view,
                                 bool is_devtools) override {
    CefWindow::CreateTopLevelWindow(
        new PopupWindowDelegate(popup_browser_view));
    return true;
  }

  cef_runtime_style_t GetBrowserRuntimeStyle() override {
    return CEF_RUNTIME_STYLE_ALLOY;
  }

 private:
  class PopupWindowDelegate : public CefWindowDelegate {
   public:
    explicit PopupWindowDelegate(CefRefPtr<CefBrowserView> popup_view)
        : popup_view_(popup_view) {}

    void OnWindowCreated(CefRefPtr<CefWindow> window) override {
      window->SetToFillLayout();
      window->AddChildView(popup_view_);
      window->Show();
    }

    void OnWindowDestroyed(CefRefPtr<CefWindow> window) override {
      popup_view_ = nullptr;
    }

    bool CanClose(CefRefPtr<CefWindow> window) override {
      CefRefPtr<CefBrowser> browser =
          popup_view_ ? popup_view_->GetBrowser() : nullptr;
      return !browser || browser->GetHost()->TryCloseBrowser();
    }

    cef_runtime_style_t GetWindowRuntimeStyle() override {
      return CEF_RUNTIME_STYLE_ALLOY;
    }

   private:
    CefRefPtr<CefBrowserView> popup_view_;

    IMPLEMENT_REFCOUNTING(PopupWindowDelegate);
  };

  IMPLEMENT_REFCOUNTING(XenonBrowserViewDelegate);
};

std::string CommandLineSwitch(const char* name) {
  CefRefPtr<CefCommandLine> command_line = CefCommandLine::GetGlobalCommandLine();
  return command_line->GetSwitchValue(name).ToString();
}

}  // namespace

XenonApp::XenonApp() = default;

void XenonApp::OnRegisterCustomSchemes(
    CefRawPtr<CefSchemeRegistrar> registrar) {
  RegisterXenonSchemes(registrar);
}

void XenonApp::OnContextInitialized() {
  CEF_REQUIRE_UI_THREAD();

  // Serve the shared UI from the directory given on the command line.
  std::string ui_dir = CommandLineSwitch("ui-dir");
  if (!ui_dir.empty()) {
    RegisterUiSchemeHandler(ui_dir);
  }

  CefBrowserSettings browser_settings;

  // The privileged UI document. Without a UI build present the view
  // stays blank rather than loading anything remote (Design.md 4.1).
  std::string ui_url = CommandLineSwitch("ui-url");
  if (ui_url.empty()) {
    ui_url = "xenon://ui/index.html";
  }
  CefRefPtr<XenonHandler> ui_handler(new XenonHandler(true));
  CefRefPtr<CefBrowserView> ui_view = CefBrowserView::CreateBrowserView(
      ui_handler, ui_url, browser_settings, nullptr, nullptr,
      new XenonBrowserViewDelegate());

  // The first tab loads the URL given on the command line (default:
  // a plain HTTPS site) so navigation is provable without any UI wiring.
  CefRefPtr<XenonHandler> tab_handler(new XenonHandler(false));
  std::string url = CommandLineSwitch("url");
  if (url.empty()) {
    url = "https://example.com";
  }
  // Empty initial URL: the load happens after overlay attachment, which
  // aborts in-flight provisional loads created before attachment.
  CefRefPtr<CefBrowserView> tab_view = CefBrowserView::CreateBrowserView(
      tab_handler, std::string(), browser_settings, nullptr, nullptr,
      new XenonBrowserViewDelegate());

  CefWindow::CreateTopLevelWindow(
      new XenonWindowDelegate(ui_view, tab_view, url));
}
