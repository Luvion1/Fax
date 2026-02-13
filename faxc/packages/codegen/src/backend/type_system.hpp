#ifndef TYPE_SYSTEM_HPP
#define TYPE_SYSTEM_HPP

#include <string>
#include <map>
#include <vector>
#include <memory>

namespace fax {
namespace codegen {

enum class TypeKind {
    INTEGER,
    FLOAT,
    BOOLEAN,
    STRING,
    ARRAY,
    STRUCT,
    FUNCTION,
    POINTER,
    VOID,
    UNKNOWN
};

struct TypeInfo {
    TypeKind kind;
    std::string name;
    std::vector<std::shared_ptr<TypeInfo>> param_types;  // For function types
    std::shared_ptr<TypeInfo> return_type;               // For function types
    std::shared_ptr<TypeInfo> element_type;              // For array/pointer types
    std::map<std::string, std::shared_ptr<TypeInfo>> fields;  // For struct types
    
    TypeInfo(TypeKind k) : kind(k) {}
    TypeInfo(TypeKind k, const std::string& n) : kind(k), name(n) {}
};

class TypeSystem {
private:
    std::map<std::string, std::shared_ptr<TypeInfo>> named_types;
    
public:
    TypeSystem();
    
    // Register a new type
    void registerType(const std::string& name, std::shared_ptr<TypeInfo> type);
    
    // Get a registered type
    std::shared_ptr<TypeInfo> getType(const std::string& name);
    
    // Create primitive types
    static std::shared_ptr<TypeInfo> getIntType();
    static std::shared_ptr<TypeInfo> getFloatType();
    static std::shared_ptr<TypeInfo> getBoolType();
    static std::shared_ptr<TypeInfo> getStringType();
    static std::shared_ptr<TypeInfo> getVoidType();
    static std::shared_ptr<TypeInfo> getUnknownType();
    
    // Create composite types
    static std::shared_ptr<TypeInfo> getArrayType(std::shared_ptr<TypeInfo> elementType);
    static std::shared_ptr<TypeInfo> getPointerType(std::shared_ptr<TypeInfo> baseType);
    static std::shared_ptr<TypeInfo> getFunctionType(
        std::vector<std::shared_ptr<TypeInfo>> paramTypes,
        std::shared_ptr<TypeInfo> returnType
    );
    static std::shared_ptr<TypeInfo> getStructType(
        const std::string& name,
        std::map<std::string, std::shared_ptr<TypeInfo>> fields
    );
    
    // Type checking utilities
    bool isAssignableFrom(std::shared_ptr<TypeInfo> targetType, std::shared_ptr<TypeInfo> sourceType);
    std::shared_ptr<TypeInfo> getCommonType(std::shared_ptr<TypeInfo> type1, std::shared_ptr<TypeInfo> type2);
    
    // Type name to string
    std::string typeToString(std::shared_ptr<TypeInfo> type);
};

}
}

#endif