// CoreRpcClient implementation: platform process spawning (Windows and
// Linux) plus a reader thread that demultiplexes JSON-RPC responses.
// The child exits when its stdin closes (Architecture.md 4.2), and this
// class mirrors that lifecycle.
#include "core_client.h"

#include <cstring>

#include "include/base/cef_logging.h"
#include "include/wrapper/cef_helpers.h"

#if defined(OS_WIN)
#include <windows.h>
#else
#include <cstdlib>
#include <unistd.h>
#endif

namespace {

constexpr size_t kMaxLineBytes = 1024 * 1024;  // Architecture.md 4.2.

// JSON-RPC ids are integers assigned by Call(); find the id in a
// response line with a plain scan — the transport must not depend on
// CEF objects (they crash when used from this plain reader thread).
bool ExtractResponseId(const std::string& line, int* id_out) {
  const std::string marker = "\"id\":";
  auto position = line.find(marker);
  if (position == std::string::npos) {
    return false;
  }
  position += marker.size();
  if (position < line.size() && line[position] == ' ') {
    position++;
  }
  int value = 0;
  bool digits = false;
  while (position < line.size() && line[position] >= '0' &&
         line[position] <= '9') {
    value = value * 10 + (line[position] - '0');
    digits = true;
    position++;
  }
  if (!digits) {
    return false;
  }
  *id_out = value;
  return true;
}

#if defined(OS_WIN)

struct PlatformHandles {
  HANDLE child_stdin_write = nullptr;
  HANDLE child_stdout_read = nullptr;
  HANDLE process = nullptr;
};

#endif

}  // namespace

struct CoreRpcClient::PlatformState {
#if defined(OS_WIN)
  PlatformHandles win;
#else
  FILE* stdin_stream = nullptr;
  int stdout_fd = -1;
  pid_t pid = -1;
#endif
};

CoreRpcClient::~CoreRpcClient() {
  Shutdown();
  if (reader_.joinable()) {
    reader_.join();
  }
  delete platform_;
}

bool CoreRpcClient::Start(const std::string& core_executable,
                          const std::string& db_path,
                          const std::string& db_key_hex,
                          const std::string& log_file) {
  DCHECK(!running_);
  platform_ = new PlatformState();
#if defined(OS_WIN)
  SECURITY_ATTRIBUTES inherit = {sizeof(SECURITY_ATTRIBUTES), nullptr, TRUE};
  HANDLE stdin_read = nullptr;
  HANDLE stdout_write = nullptr;
  if (!CreatePipe(&stdin_read, &platform_->win.child_stdin_write, &inherit, 0) ||
      !CreatePipe(&platform_->win.child_stdout_read, &stdout_write, &inherit,
                  0)) {
    return false;
  }
  SetHandleInformation(platform_->win.child_stdin_write, HANDLE_FLAG_INHERIT, 0);
  SetHandleInformation(platform_->win.child_stdout_read, HANDLE_FLAG_INHERIT, 0);

  std::wstring command = L"\"" + CefString(core_executable).ToWString() + L"\"";
  // Double-null-terminated UTF-16 environment block: start from the
  // parent environment (the child needs SystemRoot and friends to boot)
  // and overlay the core variables.
  std::wstring env;
  LPWCH parent_env = GetEnvironmentStringsW();
  if (parent_env) {
    for (wchar_t* entry = parent_env; *entry != L'\0';) {
      std::wstring line = entry;
      if (line.rfind(L"XENON_CORE_", 0) != 0) {
        env += line;
        env += L'\0';
      }
      entry += line.size() + 1;
    }
    FreeEnvironmentStringsW(parent_env);
  }
  env += L"XENON_CORE_DB_PATH=" + CefString(db_path).ToWString() + L'\0';
  env += L"XENON_CORE_DB_KEY=" + CefString(db_key_hex).ToWString() + L'\0';
  env += L"XENON_CORE_LOG_FILE=" + CefString(log_file).ToWString() + L'\0';
  env += L'\0';
  STARTUPINFOW startup = {sizeof(STARTUPINFOW)};
  // Route the child's std handles to the pipes: without
  // STARTF_USESTDHANDLES the child inherits the parent's console handles
  // and nothing written to its stdout ever reaches us.
  startup.dwFlags = STARTF_USESTDHANDLES;
  startup.hStdInput = stdin_read;
  startup.hStdOutput = stdout_write;
  startup.hStdError = stdout_write;
  PROCESS_INFORMATION process_info = {};
  std::vector<wchar_t> env_block(env.begin(), env.end());
  BOOL ok = CreateProcessW(nullptr, &command[0], nullptr, nullptr, TRUE,
                           CREATE_NO_WINDOW | CREATE_UNICODE_ENVIRONMENT,
                           env_block.data(), nullptr, &startup,
                           &process_info);
  CloseHandle(stdin_read);
  CloseHandle(stdout_write);
  if (!ok) {
    LOG(ERROR) << "xenon-blink: failed to spawn xenon-core";
    return false;
  }
  CloseHandle(process_info.hThread);
  platform_->win.process = process_info.hProcess;
#else
  int stdin_pipe[2];
  int stdout_pipe[2];
  if (pipe(stdin_pipe) != 0 || pipe(stdout_pipe) != 0) {
    return false;
  }
  pid_t pid = fork();
  if (pid < 0) {
    return false;
  }
  if (pid == 0) {
    // Child: wire the pipes to stdin/stdout and exec the core.
    dup2(stdin_pipe[0], STDIN_FILENO);
    dup2(stdout_pipe[1], STDOUT_FILENO);
    close(stdin_pipe[0]);
    close(stdin_pipe[1]);
    close(stdout_pipe[0]);
    close(stdout_pipe[1]);
    setenv("XENON_CORE_DB_PATH", db_path.c_str(), 1);
    setenv("XENON_CORE_DB_KEY", db_key_hex.c_str(), 1);
    setenv("XENON_CORE_LOG_FILE", log_file.c_str(), 1);
    execl(core_executable.c_str(), core_executable.c_str(), (char*)nullptr);
    _exit(127);  // exec failed.
  }
  close(stdin_pipe[0]);
  close(stdout_pipe[1]);
  platform_->stdin_stream = fdopen(stdin_pipe[1], "w");
  platform_->stdout_fd = stdout_pipe[0];
  platform_->pid = pid;
#endif
  running_ = true;
  reader_ = std::thread(&CoreRpcClient::ReaderLoop, this);
  return true;
}

void CoreRpcClient::Call(const std::string& method,
                         const std::string& params_json,
                         ResponseCallback callback) {
  int id;
  {
    std::lock_guard<std::mutex> lock(pending_mutex_);
    id = next_id_++;
    pending_[id] = std::move(callback);
  }
  std::string request = "{\"jsonrpc\":\"2.0\",\"id\":" + std::to_string(id) +
                        ",\"method\":\"" + method + "\"";
  if (!params_json.empty()) {
    request += ",\"params\":" + params_json;
  }
  request += "}";
  SendLine(request);
}

void CoreRpcClient::Shutdown() {
  if (!running_) {
    return;
  }
  running_ = false;
#if defined(OS_WIN)
  if (platform_->win.child_stdin_write) {
    CloseHandle(platform_->win.child_stdin_write);
    platform_->win.child_stdin_write = nullptr;
  }
#else
  if (platform_->stdin_stream) {
    fclose(platform_->stdin_stream);
    platform_->stdin_stream = nullptr;
  }
#endif
  // Failing pending callers: the pipe is closed, no responses will come.
  std::map<int, ResponseCallback> abandoned;
  {
    std::lock_guard<std::mutex> lock(pending_mutex_);
    abandoned.swap(pending_);
  }
  for (auto& entry : abandoned) {
    entry.second(false, "{}");
  }
}

bool CoreRpcClient::IsRunning() const {
  return running_;
}

void CoreRpcClient::SendLine(const std::string& line) {
  std::lock_guard<std::mutex> lock(write_mutex_);
#if defined(OS_WIN)
  DWORD written = 0;
  HANDLE handle = platform_->win.child_stdin_write;
  if (!handle || !WriteFile(handle, line.data(),
                            static_cast<DWORD>(line.size()), &written,
                            nullptr) ||
      !WriteFile(handle, "\n", 1, &written, nullptr)) {
    running_ = false;
  }
#else
  if (!platform_->stdin_stream) {
    running_ = false;
    return;
  }
  if (fputs(line.c_str(), platform_->stdin_stream) == EOF ||
      fputc('\n', platform_->stdin_stream) == EOF ||
      fflush(platform_->stdin_stream) != 0) {
    running_ = false;
  }
#endif
}

void CoreRpcClient::ReaderLoop() {
  std::string line;
  line.reserve(4096);
  char chunk[4096];
  while (running_) {
#if defined(OS_WIN)
    DWORD read = 0;
    if (!ReadFile(platform_->win.child_stdout_read, chunk, sizeof(chunk),
                  &read, nullptr) ||
        read == 0) {
      break;
    }
#else
    ssize_t read = ::read(platform_->stdout_fd, chunk, sizeof(chunk));
    if (read <= 0) {
      break;
    }
#endif
    for (size_t i = 0; i < static_cast<size_t>(read); i++) {
      if (chunk[i] == '\n') {
        int id = -1;
        if (!line.empty() && ExtractResponseId(line, &id)) {
          ResponseCallback callback;
          {
            std::lock_guard<std::mutex> lock(pending_mutex_);
            auto it = pending_.find(id);
            if (it != pending_.end()) {
              callback = std::move(it->second);
              pending_.erase(it);
            }
          }
          if (callback) {
            callback(true, line);
          }
        }
        line.clear();
      } else if (line.size() < kMaxLineBytes) {
        line.push_back(chunk[i]);
      }
    }
  }
  running_ = false;
}
