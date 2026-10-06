// Xenon Blink host entry point, shared by Windows and Linux. CEF
// sub-processes (render, GPU, ...) re-enter through this executable.
//
// Sandbox note (SEC-01): no_sandbox is never set. On Windows the build
// links cef_sandbox.lib and passes the sandbox information to CEF; on
// Linux the chrome-sandbox helper is expected next to the executable.
#include <string>

#include "xenon_app.h"
#include "include/cef_command_line.h"

#if defined(OS_WIN)
#include <windows.h>

#include "include/cef_sandbox_win.h"
#include "include/cef_version_info.h"

namespace {

int RunMain(HINSTANCE hInstance, void* sandbox_info) {
  CefMainArgs main_args(hInstance);

  // Sub-process dispatch: returns >= 0 when this invocation is a
  // render/GPU/utility child process.
  int exit_code = CefExecuteProcess(main_args, nullptr, sandbox_info);
  if (exit_code >= 0) {
    return exit_code;
  }

  CefRefPtr<CefCommandLine> command_line = CefCommandLine::CreateCommandLine();
  command_line->InitFromString(::GetCommandLineW());

  CefSettings settings;
  CefRefPtr<XenonApp> app(new XenonApp);

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
  CefMainArgs main_args(argc, argv);

  int exit_code = CefExecuteProcess(main_args, nullptr, nullptr);
  if (exit_code >= 0) {
    return exit_code;
  }

  // Non-fatal X errors must not terminate the process.
  XSetErrorHandler(XErrorHandlerImpl);
  XSetIOErrorHandler(XIOErrorHandlerImpl);

  CefSettings settings;
  CefRefPtr<XenonApp> app(new XenonApp);

  if (!CefInitialize(main_args, settings, app.get(), nullptr)) {
    return CefGetExitCode();
  }
  CefRunMessageLoop();
  CefShutdown();
  return 0;
}

#endif  // OS_WIN / OS_LINUX
