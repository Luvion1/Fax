#ifndef FAX_PROTO_FFI_H
#define FAX_PROTO_FFI_H

#include <cstdint>
#include <cstddef>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque handles for Lean
typedef void* ProtoContext;
typedef void* ProtoMessage;

// Context management
ProtoContext fax_proto_context_new();
void fax_proto_context_free(ProtoContext ctx);
const char* fax_proto_get_error(ProtoContext ctx);

// Token serialization
uint8_t* fax_serialize_tokens(ProtoContext ctx, const char* source, size_t* out_size);
void fax_bytes_free(uint8_t* bytes);

// Token deserialization
int fax_deserialize_tokens(ProtoContext ctx, const uint8_t* data, size_t size);
int fax_get_token_count(ProtoContext ctx);
void fax_get_token_info(ProtoContext ctx, int index, int* type, const char** text, int* line, int* col);

// AST serialization
uint8_t* fax_serialize_module(ProtoContext ctx, const char* ast_json, size_t* out_size);
int fax_deserialize_module(ProtoContext ctx, const uint8_t* data, size_t size, char** out_json);
void fax_string_free(char* str);

// Utility
const char* fax_proto_version();

#ifdef __cplusplus
}
#endif

#endif // FAX_PROTO_FFI_H
