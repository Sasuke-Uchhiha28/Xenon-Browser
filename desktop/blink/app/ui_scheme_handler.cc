// Implementation of the xenon://ui file server. See ui_scheme_handler.h.
// Every request is validated (SEC-02): only paths inside the UI root are
// served, traversal is rejected, and nothing about the request content
// is logged (PRIV-05 — request paths are not logged).
#include "ui_scheme_handler.h"

#include <algorithm>
#include <filesystem>
#include <string>

#include "include/base/cef_logging.h"
#include "include/base/cef_macros.h"
#include "include/cef_parser.h"
#include "include/cef_scheme.h"
#include "include/wrapper/cef_helpers.h"

namespace {

namespace fs = std::filesystem;

// Set once during registration, before any request can arrive, and read
// from the IO thread afterwards. Registration happens on the UI thread
// during startup; there is no other writer, so no lock is needed.
fs::path g_ui_root;

// Map the request path to a file under the UI root, or empty when the
// request must be refused. "/" serves index.html (the SPA entry point).
std::string ResolveUiPath(const std::string& url_path) {
  if (g_ui_root.empty()) {
    return std::string();
  }
  std::string relative = url_path;
  if (relative.empty() || relative == "/") {
    relative = "index.html";
  }
  // Reject traversal and absolute-looking paths before joining.
  if (relative.find("..") != std::string::npos ||
      relative.find('\\') != std::string::npos ||
      relative.rfind('/', 0) == 0) {
    return std::string();
  }
  fs::path candidate = g_ui_root / fs::path(relative);
  // Defense in depth: the resolved file must stay inside the root.
  auto root_canonical = std::error_code();
  auto canonical = fs::weakly_canonical(candidate, root_canonical);
  auto root = fs::weakly_canonical(g_ui_root, root_canonical);
  auto root_string = root.string();
  auto candidate_string = canonical.string();
  if (candidate_string.compare(0, root_string.size(), root_string) != 0) {
    return std::string();
  }
  std::error_code exists;
  if (!fs::is_regular_file(canonical, exists)) {
    return std::string();
  }
  return canonical.string();
}

class UiResourceHandler : public CefResourceHandler {
 public:
  explicit UiResourceHandler(const std::string& ui_root)
      : ui_root_(ui_root) {}

  bool Open(CefRefPtr<CefRequest> request,
            bool& handle_request,
            CefRefPtr<CefCallback> callback) override {
    CEF_REQUIRE_IO_THREAD();
    // The URL path after xenon://ui, URL-decoded.
    CefURLParts parts;
    const std::string& url = request->GetURL();
    if (!CefParseURL(url, parts)) {
      return false;  // Malformed request: refuse.
    }
    CefString path_string(&parts.path);
    std::string file = ResolveUiPath(path_string.ToString());
    if (file.empty()) {
      status_code_ = 404;
      status_text_ = "Not Found";
      content_ = "<html><body>404</body></html>";
      mime_type_ = "text/html";
      handle_request = true;
      return true;
    }
    stream_ = CefStreamReader::CreateForFile(file);
    if (!stream_) {
      status_code_ = 404;
      status_text_ = "Not Found";
      content_ = "<html><body>404</body></html>";
      mime_type_ = "text/html";
      handle_request = true;
      return true;
    }
    // Guess the content type from the extension.
    std::string extension = fs::path(file).extension().string();
    if (!extension.empty() && extension.starts_with('.')) {
      extension = extension.substr(1);
    }
    CefString mime = CefGetMimeType(extension);
    mime_type_ = mime.empty() ? "application/octet-stream" : mime.ToString();
    status_code_ = 200;
    status_text_ = "OK";
    handle_request = true;
    return true;
  }

  void GetResponseHeaders(CefRefPtr<CefResponse> response,
                          int64_t& response_length,
                          CefString& redirectUrl) override {
    response->SetStatus(status_code_);
    response->SetStatusText(status_text_);
    response->SetMimeType(mime_type_);
    // Privileged UI document: a strict CSP; the React app needs inline
    // styles for attribute styling but loads everything else locally.
    response->SetHeaderByName("Content-Security-Policy",
                              "default-src 'self'; script-src 'self'; "
                              "style-src 'self' 'unsafe-inline'; "
                              "img-src 'self' data:; font-src 'self'; "
                              "connect-src 'self'; object-src 'none'; "
                              "base-uri 'none'; form-action 'none'",
                              false);
    if (!content_.empty()) {
      response_length = static_cast<int64_t>(content_.size());
    } else if (stream_) {
      // Unknown length up front; ReadResponse signals the end with 0.
      response_length = -1;
    } else {
      response_length = 0;
    }
  }

  bool ReadResponse(void* data_out,
                    int bytes_to_read,
                    int& bytes_read,
                    CefRefPtr<CefCallback> callback) override {
    CEF_REQUIRE_IO_THREAD();
    if (!content_.empty()) {
      // Inline body (404 pages): serve once, then end the stream.
      bytes_read = static_cast<int>(
          std::min(content_.size(), static_cast<size_t>(bytes_to_read)));
      memcpy(data_out, content_.data(), bytes_read);
      content_.erase(0, bytes_read);
      return true;
    }
    if (!stream_) {
      bytes_read = 0;
      return true;
    }
    bytes_read = static_cast<int>(
        stream_->Read(data_out, 1, static_cast<size_t>(bytes_to_read)));
    return true;  // Synchronous reads; the files are small and local.
  }

  void Cancel() override { stream_ = nullptr; }

 private:
  std::string ui_root_;  // Unused after construction; kept for clarity.
  CefRefPtr<CefStreamReader> stream_;
  std::string content_;
  std::string mime_type_;
  std::string status_text_;
  int status_code_ = 200;

  IMPLEMENT_REFCOUNTING(UiResourceHandler);
  DISALLOW_COPY_AND_ASSIGN(UiResourceHandler);
};

class UiSchemeFactory : public CefSchemeHandlerFactory {
 public:
  explicit UiSchemeFactory(const std::string& ui_root) : ui_root_(ui_root) {}

  CefRefPtr<CefResourceHandler> Create(
      CefRefPtr<CefBrowser> browser,
      CefRefPtr<CefFrame> frame,
      const CefString& scheme_name,
      CefRefPtr<CefRequest> request) override {
    return new UiResourceHandler(ui_root_);
  }

 private:
  std::string ui_root_;

  IMPLEMENT_REFCOUNTING(UiSchemeFactory);
  DISALLOW_COPY_AND_ASSIGN(UiSchemeFactory);
};

}  // namespace

void RegisterXenonSchemes(CefRawPtr<CefSchemeRegistrar> registrar) {
  // STANDARD makes relative URLs and module scripts behave; CORS_ENABLED
  // and FETCH_ENABLED let the Vite output (ES modules) load.
  registrar->AddCustomScheme(
      "xenon", CEF_SCHEME_OPTION_STANDARD | CEF_SCHEME_OPTION_CORS_ENABLED |
                   CEF_SCHEME_OPTION_FETCH_ENABLED);
}

void RegisterUiSchemeHandler(const std::string& ui_root) {
  CEF_REQUIRE_UI_THREAD();
  g_ui_root = ui_root;
  if (!CefRegisterSchemeHandlerFactory("xenon", "ui",
                                       new UiSchemeFactory(ui_root))) {
    LOG(ERROR) << "xenon-blink: failed to register the xenon://ui handler";
  }
}
