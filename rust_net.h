#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

/// client context
struct ClientContext;

/// tokio context
struct TokioContext;

struct RequestResponse {
  const uint8_t *data;
  uintptr_t len;
  uint32_t status;
  int32_t version;
};

extern "C" {

TokioContext *rust_net_tokio_new(uint32_t thread_count);

void rust_net_tokio_free(TokioContext *handler);

ClientContext *rust_net_client_new(bool brotli, bool cookie_store);

void rust_net_client_free(ClientContext *handler);

void rust_net_add_header(ClientContext *context, const char *key, const char *value);

void rust_net_clear_header(ClientContext *context);

void rust_net_add_param(ClientContext *context, const char *key, const char *value);

void rust_net_set_clear_expires_enabled(ClientContext *context, bool value);

void rust_net_clear_param(ClientContext *context);

uint64_t rust_net_post(TokioContext *tokio_context,
                       ClientContext *client_context,
                       const char *url,
                       const uint8_t *data,
                       uintptr_t length);

uint64_t rust_net_get(TokioContext *tokio_context, ClientContext *client_context, const char *url);

void rust_net_remove_request(ClientContext *client_context, uint64_t key);

/// 获取reqwest请求状态
/// 0正在请求
/// -1请求失败
/// 1请求成功
/// -2请求不存在
int32_t rust_net_get_request_state(ClientContext *client_context, uint64_t key);

/// 获取reqwest请求结果中的错误信息
/// 使用完成之后 调用 rust_net_free_string 释放内存
char *rust_net_get_request_error(ClientContext *client_context, uint64_t key);

/// 获取reqwest请求结果
/// 使用完成之后 调用 rust_net_free_request_response 释放内存
RequestResponse rust_net_get_request_response(ClientContext *client_context, uint64_t key);

void rust_net_free_string(char *s);

void rust_net_free_request_response(RequestResponse resp);

} // extern "C"
