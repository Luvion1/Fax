#include "ffi.h"
#include <string>
#include <vector>
#include <memory>
#include <cstring>

// Forward declarations for generated protobuf classes
namespace fax {
namespace compiler {
class TokenStream;
class Module;
class CompilerError;
}
}

// Context to hold protobuf state
struct ProtoContextData {
    std::string error;
    std::vector<uint8_t> buffer;
    std::string temp_string;
    
    // Store deserialized messages
    std::unique_ptr<fax::compiler::TokenStream> token_stream;
    std::unique_ptr<fax::compiler::Module> module;
};

extern "C" {

ProtoContext fax_proto_context_new() {
    return new ProtoContextData();
}

void fax_proto_context_free(ProtoContext ctx) {
    delete static_cast<ProtoContextData*>(ctx);
}

const char* fax_proto_get_error(ProtoContext ctx) {
    auto* data = static_cast<ProtoContextData*>(ctx);
    if (!data || data->error.empty()) {
        return nullptr;
    }
    data->temp_string = data->error;
    return data->temp_string.c_str();
}

// Serialize tokens (stub - will be implemented with actual protobuf)
uint8_t* fax_serialize_tokens(ProtoContext ctx, const char* source, size_t* out_size) {
    auto* data = static_cast<ProtoContextData*>(ctx);
    if (!data || !source || !out_size) {
        return nullptr;
    }
    
    // TODO: Implement actual protobuf serialization
    // For now, return empty buffer as placeholder
    data->error = "Token serialization not yet implemented with protobuf";
    data->buffer.clear();
    *out_size = 0;
    return nullptr;
}

void fax_bytes_free(uint8_t* bytes) {
    // Memory is managed by context, no-op here
    (void)bytes;
}

// Deserialize tokens (stub)
int fax_deserialize_tokens(ProtoContext ctx, const uint8_t* data, size_t size) {
    auto* ctx_data = static_cast<ProtoContextData*>(ctx);
    if (!ctx_data || !data || size == 0) {
        return -1;
    }
    
    // TODO: Implement actual protobuf deserialization
    ctx_data->error = "Token deserialization not yet implemented";
    return -1;
}

int fax_get_token_count(ProtoContext ctx) {
    auto* data = static_cast<ProtoContextData*>(ctx);
    if (!data || !data->token_stream) {
        return 0;
    }
    return 0; // TODO: Return actual count
}

void fax_get_token_info(ProtoContext ctx, int index, int* type, const char** text, int* line, int* col) {
    auto* data = static_cast<ProtoContextData*>(ctx);
    (void)data;
    (void)index;
    
    // Set defaults
    if (type) *type = 0;
    if (text) *text = "";
    if (line) *line = 0;
    if (col) *col = 0;
    
    // TODO: Implement actual token info retrieval
}

// AST serialization (stub)
uint8_t* fax_serialize_module(ProtoContext ctx, const char* ast_json, size_t* out_size) {
    auto* data = static_cast<ProtoContextData*>(ctx);
    if (!data || !ast_json || !out_size) {
        return nullptr;
    }
    
    // TODO: Implement actual protobuf serialization
    data->error = "Module serialization not yet implemented";
    *out_size = 0;
    return nullptr;
}

int fax_deserialize_module(ProtoContext ctx, const uint8_t* data, size_t size, char** out_json) {
    auto* ctx_data = static_cast<ProtoContextData*>(ctx);
    if (!ctx_data || !data || size == 0 || !out_json) {
        return -1;
    }
    
    // TODO: Implement actual protobuf deserialization
    ctx_data->error = "Module deserialization not yet implemented";
    return -1;
}

void fax_string_free(char* str) {
    delete[] str;
}

const char* fax_proto_version() {
    return "Fax Protobuf FFI v0.0.3";
}

} // extern "C"
