// Xenon Blink host entry point, shared by Windows and Linux. CEF
// sub-processes (render, GPU, ...) re-enter through this executable.
//
// Sandbox note (SEC-01): no_sandbox is never set. On Windows the build
// links cef_sandbox.lib and passes the sandbox information to CEF; on
// Linux the chrome-sandbox helper is expected next to the executable.
#include <chrono>
#include <filesystem>
#include <iostream>
#include <string>
#include <thread>

#include "core_client.h"
#include "xenon_app.h"
#include "include/cef_command_line.h"

namespace {

// --check-core: spawn the core, round-trip a hello and a settings
// write/read, print evidence, exit 0/1. Runs BEFORE any CEF setup so it
// works headless (CI) — no window, no engine. Returns -1 when the flag
// is absent (normal startup continues).
int RunCheckCore() {
  // Windows detects the flag from the command line (wWinMain has no
  // argv); Linux checks argv in main() and only calls this when present.
  bool requested = false;
#if defined(OS_WIN)
  requested = std::string(::GetCommandLineA()).find("--check-core") !=
              std::string::npos;
#endif
  if (!requested) {
    return -1;
  }

  CoreRpcClient client;
  auto db = std::filesystem::temp_directory_path() / "xenon-check-core.db";
  std::error_code ignored;
  std::filesystem::remove(db, ignored);
  std::string core = "xenon-core";
#if defined(OS_WIN)
  core += ".exe";
#endif
  if (!client.Start(core, db.string(), std::string(64, 'a'), "")) {
    std::cout << "check-core FAILED: could not spawn " << core << std::endl;
    return 1;
  }

  std::string hello_response;
  client.Call("hello", "", [&](bool ok, const std::string& response) {
    hello_response = ok ? response : "";
  });
  int attempts = 0;
  while (hello_response.empty() && attempts++ < 50) {
    std::this_thread::sleep_for(std::chrono::milliseconds(100));
  }
  if (hello_response.find("\"protocolVersion\":1") == std::string::npos) {
    std::cout << "check-core FAILED: bad hello: " << hello_response
              << std::endl;
    client.Shutdown();
    return 1;
  }

  client.Call("settings.set", "{\"key\":\"check\",\"value\":\"ok\"}",
              [](bool, const std::string&) {});
  std::string get_response;
  client.Call("settings.get", "{\"key\":\"check\"}",
              [&](bool ok, const std::string& response) {
                get_response = ok ? response : "";
              });
  attempts = 0;
  while (get_response.empty() && attempts++ < 50) {
    std::this_thread::sleep_for(std::chrono::milliseconds(100));
  }
  client.Shutdown();
  if (get_response.find("\"value\":\"ok\"") == std::string::npos) {
    std::cout << "check-core FAILED: settings round trip: " << get_response
              << std::endl;
    return 1;
  }
  std::filesystem::remove(db, ignored);
  std::cout << "check-core OK: hello + settings round trip verified"
            << std::endl;
  return 0;
}

}  // namespace

#if defined(OS_WIN)
#include <windows.h>

#include "include/cef_sandbox_win.h"
#include "include/cef_version_info.h"

namespace {

int RunMain(HINSTANCE hInstance, void* sandbox_info) {
  int check = RunCheckCore();
  if (check >= 0) {
    return check;
  }

  CefMainArgs main_args(hInstance);

  // The app must reach CHILD processes too: custom schemes are registered
  // via CefApp::OnRegisterCustomSchemes in every process, and the
  // renderer refuses to commit URLs whose scheme it does not know.
  CefRefPtr<XenonApp> app(new XenonApp);

  // Sub-process dispatch: returns >= 0 when this invocation is a
  // render/GPU/utility child process.
  int exit_code = CefExecuteProcess(main_args, app, sandbox_info);
  if (exit_code >= 0) {
    return exit_code;
  }

  CefSettings settings;
  settings.log_severity = LOGSEVERITY_WARNING;
  CefString(&settings.log_file).FromASCII("xenon-blink.log");

  if (!CefInitialize(main_args, settings, app.get(), sandbox_info)) {
    return CefGetExitCode();
  }
  CefRunMessageLoop();
  CefShutdown();
  return 0;
}

}  // namespace

#if defined(CEF_USE_BOOTSTRAP)

// Built as a DLL: bootstrap.exe loads us and calls this entry point.
CEF_BOOTSTRAP_EXPORT int RunWinMain(HINSTANCE hInstance,
                                    LPWSTR lpCmdLine,
                                    int nCmdShow,
                                    void* sandbox_info,
                                    cef_version_info_t* /*version_info*/) {
  return ::RunMain(hInstance, sandbox_info);
}

#else  // !defined(CEF_USE_BOOTSTRAP)

// Entry point function for all processes.
int APIENTRY wWinMain(HINSTANCE hInstance,
                      HINSTANCE hPrevInstance,
                      LPWSTR lpCmdLine,
                      int nCmdShow) {
  UNREFERENCED_PARAMETER(hPrevInstance);
  UNREFERENCED_PARAMETER(lpCmdLine);

#if defined(CEF_USE_SANDBOX)
  // The sandbox information must outlive CefInitialize (see
  // cef_sandbox_win.h).
  CefScopedSandboxInfo scoped_sandbox;
  void* sandbox_info = scoped_sandbox.sandbox_info();
#else
  void* sandbox_info = nullptr;
#endif

  return ::RunMain(hInstance, sandbox_info);
}

#endif  // !defined(CEF_USE_BOOTSTRAP)

#else  // OS_LINUX

#include <X11/Xlib.h>

#include "include/base/cef_logging.h"

namespace {

int XErrorHandlerImpl(Display* display, XErrorEvent* event) {
  LOG(WARNING) << "xenon-blink: X error (type " << event->type
               << ", request_code " << event->request_code << ")";
  return 0;
}

int XIOErrorHandlerImpl(Display* display) {
  return 0;
}

}  // namespace

int main(int argc, char* argv[]) {
  for (int i = 1; i < argc; i++) {
    if (std::string(argv[i]) == "--check-core") {
      return RunCheckCore();
    }
  }

  CefMainArgs main_args(argc, argv);

  // The app must reach CHILD processes too (see the Windows note).
  CefRefPtr<XenonApp> app(new XenonApp);

  int exit_code = CefExecuteProcess(main_args, app, nullptr);
  if (exit_code >= 0) {
    return exit_code;
  }

  // Non-fatal X errors must not terminate the process.
  XSetErrorHandler(XErrorHandlerImpl);
  XSetIOErrorHandler(XIOErrorHandlerImpl);

  CefSettings settings;

  if (!CefInitialize(main_args, settings, app.get(), nullptr)) {
    return CefGetExitCode();
  }
  CefRunMessageLoop();
  CefShutdown();
  return 0;
}

#endif  // OS_WIN / OS_LINUX
