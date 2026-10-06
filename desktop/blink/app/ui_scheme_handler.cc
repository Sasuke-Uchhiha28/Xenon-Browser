// Implementation of the xenon://ui file server. See ui_scheme_handler.h.
// Every request is validated (SEC-02): only paths inside the UI root are
// served, traversal is rejected, and nothing about the request content
// is logged (PRIV-05 — request paths are not logged).
#include "ui_scheme_handler.h"

#include <algorithm>
#include <filesystem>
#include <fstream>
#include <sstream>
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
// request must be refused. "/" and unknown extension-less paths serve
// index.html (the SPA entry point).
std::string ResolveUiPath(const std::string& url_path) {
  if (g_ui_root.empty()) {
    return std::string();
  }
  std::string relative = url_path;
  // URL paths always start with "/" — strip it.
  while (!relative.empty() && relative[0] == '/') {
    relative.erase(0, 1);
  }
  // Reject traversal and Windows-style paths before joining.
  if (relative.find("..") != std::string::npos ||
      relative.find('\\') != std::string::npos) {
    return std::string();
  }
  if (relative.empty()) {
    relative = "index.html";
  }
  fs::path candidate = g_ui_root / fs::path(relative);
  // Defense in depth: the resolved file must stay inside the root.
  std::error_code ec;
  auto canonical = fs::weakly_canonical(candidate, ec);
  auto root = fs::weakly_canonical(g_ui_root, ec);
  if (canonical.string().compare(0, root.string().size(), root.string()) !=
      0) {
    return std::string();
  }
  if (!fs::is_regular_file(canonical, ec)) {
    // SPA fallback: paths without an extension are client-side routes.
    if (fs::path(relative).extension().empty()) {
      auto index = g_ui_root / "index.html";
      if (fs::is_regular_file(index, ec)) {
        return index.string();
      }
    }
    return std::string();
  }
  return canonical.string();
}

// Serves the shared UI from memory: UI files are small (the largest is
// the JS bundle at a few hundred KB), so Open() reads the file once and
// ReadResponse() hands out slices. This mirrors the CEF reference
// handler contract: a KNOWN response length and a sync ReadResponse that
// returns false once every byte has been delivered.
class UiResourceHandler : public CefResourceHandler {
 public:
  UiResourceHandler() = default;

  bool Open(CefRefPtr<CefRequest> request,
            bool& handle_request,
            CefRefPtr<CefCallback> callback) override {
    CEF_REQUIRE_IO_THREAD();
    CefURLParts parts;
    if (!CefParseURL(request->GetURL().ToString(), parts)) {
      return false;  // Malformed request: refuse.
    }
    CefString path_string(&parts.path);
    std::string file = ResolveUiPath(path_string.ToString());
    if (file.empty()) {
      content_ = "<html><body>404</body></html>";
      mime_type_ = "text/html";
      status_code_ = 404;
      status_text_ = "Not Found";
    } else {
      std::ifstream in(file, std::ios::binary);
      std::ostringstream buffer;
      buffer << in.rdbuf();
      content_ = buffer.str();
      std::string extension = fs::path(file).extension().string();
      if (!extension.empty() && extension[0] == '.') {
        extension = extension.substr(1);
      }
      CefString mime = CefGetMimeType(extension);
      mime_type_ =
          mime.empty() ? "application/octet-stream" : mime.ToString();
      status_code_ = 200;
      status_text_ = "OK";
    }
    handle_request = true;
    return true;
  }

  void GetResponseHeaders(CefRefPtr<CefResponse> response,
                          int64_t& response_length,
                          CefString& redirectUrl) override {
    CEF_REQUIRE_IO_THREAD();
    response->SetStatus(status_code_);
    response->SetStatusText(status_text_);
    response->SetMimeType(mime_type_);
    response_length = static_cast<int64_t>(content_.size());
  }

  bool ReadResponse(void* data_out,
                    int bytes_to_read,
                    int& bytes_read,
                    CefRefPtr<CefCallback> callback) override {
    CEF_REQUIRE_IO_THREAD();
    bytes_read = static_cast<int>(
        std::min(content_.size(), static_cast<size_t>(bytes_to_read)));
    memcpy(data_out, content_.data(), bytes_read);
    content_.erase(0, bytes_read);
    // With the known length set above, the loader stops counting once
    // every byte is delivered; returning false at exhaustion matches the
    // reference handler.
    return bytes_read > 0;
  }

  void Cancel() override {}

 private:
  std::string content_;
  std::string mime_type_;
  std::string status_text_;
  int status_code_ = 200;

  IMPLEMENT_REFCOUNTING(UiResourceHandler);
  DISALLOW_COPY_AND_ASSIGN(UiResourceHandler);
};

class UiSchemeFactory : public CefSchemeHandlerFactory {
 public:
  UiSchemeFactory() = default;

  CefRefPtr<CefResourceHandler> Create(
      CefRefPtr<CefBrowser> browser,
      CefRefPtr<CefFrame> frame,
      const CefString& scheme_name,
      CefRefPtr<CefRequest> request) override {
    return new UiResourceHandler();
  }

 private:
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
                                       new UiSchemeFactory())) {
    LOG(ERROR) << "xenon-blink: failed to register the xenon://ui handler";
  }
}
