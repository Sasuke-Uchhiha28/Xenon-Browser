// JSON-RPC client for the xenon-core child process (Architecture.md 4.2):
// one message per line on the child's stdin/stdout, responses matched by
// id. Transport-only — all logic stays in core or the callers.
#ifndef XENON_BLINK_APP_CORE_CLIENT_H_
#define XENON_BLINK_APP_CORE_CLIENT_H_

#include <functional>
#include <map>
#include <mutex>
#include <string>
#include <thread>

// Spawns and manages one xenon-core child process. Methods are safe to
// call from any thread; callbacks run on the reader thread. Owned by the
// caller (keep it alive while requests are in flight).
class CoreRpcClient {
 public:
  using ResponseCallback =
      std::function<void(bool success, const std::string& response_json)>;

  CoreRpcClient() = default;

  CoreRpcClient(const CoreRpcClient&) = delete;
  CoreRpcClient& operator=(const CoreRpcClient&) = delete;

  ~CoreRpcClient();

  // Spawn the core executable and start reading its responses. |env|
  // entries are added to the child environment (db path, dev key).
  // Returns false when the process could not be spawned.
  bool Start(const std::string& core_executable,
             const std::string& db_path,
             const std::string& db_key_hex,
             const std::string& log_file);

  // Send one JSON-RPC request; |callback| fires with the raw response
  // line (or success=false when the pipe broke). Fire-and-forget: the
  // caller matches/validates the content.
  void Call(const std::string& method,
            const std::string& params_json,
            ResponseCallback callback);

  // Close the child's stdin: the core exits when stdin closes.
  void Shutdown();

  bool IsRunning() const;

 private:
  struct PlatformState;
  void ReaderLoop();
  void SendLine(const std::string& line);

  // Platform handle storage, owned here (see core_client.cc).
  PlatformState* platform_ = nullptr;
  std::thread reader_;
  std::mutex write_mutex_;
  std::mutex pending_mutex_;
  std::map<int, ResponseCallback> pending_;
  int next_id_ = 1;
  bool running_ = false;
};

#endif  // XENON_BLINK_APP_CORE_CLIENT_H_
