#include "type_system.hpp"
#include <memory>

namespace fax {
namespace codegen {

TypeSystem::TypeSystem() {
    // Register primitive types
    registerType("i64", getIntType());
    registerType("f64", getFloatType());
    registerType("bool", getBoolType());
    registerType("str", getStringType());
    registerType("void", getVoidType());
}

void TypeSystem::registerType(const std::string& name, std::shared_ptr<TypeInfo> type) {
    named_types[name] = type;
}

std::shared_ptr<TypeInfo> TypeSystem::getType(const std::string& name) {
    auto it = named_types.find(name);
    if (it != named_types.end()) {
        return it->second;
    }
    return getUnknownType();
}

std::shared_ptr<TypeInfo> TypeSystem::getIntType() {
    auto type = std::make_shared<TypeInfo>(TypeKind::INTEGER);
    type->name = "i64";
    return type;
}

std::shared_ptr<TypeInfo> TypeSystem::getFloatType() {
    auto type = std::make_shared<TypeInfo>(TypeKind::FLOAT);
    type->name = "f64";
    return type;
}

std::shared_ptr<TypeInfo> TypeSystem::getBoolType() {
    auto type = std::make_shared<TypeInfo>(TypeKind::BOOLEAN);
    type->name = "bool";
    return type;
}

std::shared_ptr<TypeInfo> TypeSystem::getStringType() {
    auto type = std::make_shared<TypeInfo>(TypeKind::STRING);
    type->name = "str";
    return type;
}

std::shared_ptr<TypeInfo> TypeSystem::getVoidType() {
    auto type = std::make_shared<TypeInfo>(TypeKind::VOID);
    type->name = "void";
    return type;
}

std::shared_ptr<TypeInfo> TypeSystem::getUnknownType() {
    auto type = std::make_shared<TypeInfo>(TypeKind::UNKNOWN);
    type->name = "unknown";
    return type;
}

std::shared_ptr<TypeInfo> TypeSystem::getArrayType(std::shared_ptr<TypeInfo> elementType) {
    auto type = std::make_shared<TypeInfo>(TypeKind::ARRAY);
    type->element_type = elementType;
    type->name = elementType->name + "[]";
    return type;
}

std::shared_ptr<TypeInfo> TypeSystem::getPointerType(std::shared_ptr<TypeInfo> baseType) {
    auto type = std::make_shared<TypeInfo>(TypeKind::POINTER);
    type->element_type = baseType;
    type->name = baseType->name + "*";
    return type;
}

std::shared_ptr<TypeInfo> TypeSystem::getFunctionType(
    std::vector<std::shared_ptr<TypeInfo>> paramTypes,
    std::shared_ptr<TypeInfo> returnType
) {
    auto type = std::make_shared<TypeInfo>(TypeKind::FUNCTION);
    type->param_types = paramTypes;
    type->return_type = returnType;
    // Create a name representation
    type->name = "(";
    for (size_t i = 0; i < paramTypes.size(); ++i) {
        if (i > 0) type->name += ", ";
        type->name += paramTypes[i]->name;
    }
    type->name += ") -> " + returnType->name;
    return type;
}

std::shared_ptr<TypeInfo> TypeSystem::getStructType(
    const std::string& name,
    std::map<std::string, std::shared_ptr<TypeInfo>> fields
) {
    auto type = std::make_shared<TypeInfo>(TypeKind::STRUCT);
    type->name = name;
    type->fields = fields;
    return type;
}

bool TypeSystem::isAssignableFrom(std::shared_ptr<TypeInfo> targetType, std::shared_ptr<TypeInfo> sourceType) {
    if (targetType->kind == sourceType->kind) {
        // Same kind, check if it's compatible
        if (targetType->kind == TypeKind::INTEGER || targetType->kind == TypeKind::FLOAT) {
            // For now, assume numeric types are compatible
            return true;
        } else if (targetType->kind == TypeKind::POINTER && sourceType->kind == TypeKind::POINTER) {
            // Pointer compatibility depends on base type compatibility
            return isAssignableFrom(targetType->element_type, sourceType->element_type);
        } else if (targetType->kind == TypeKind::ARRAY && sourceType->kind == TypeKind::ARRAY) {
            // Array compatibility depends on element type compatibility
            return isAssignableFrom(targetType->element_type, sourceType->element_type);
        } else {
            return targetType->name == sourceType->name;
        }
    }
    
    // Check for implicit conversions
    if (targetType->kind == TypeKind::FLOAT && sourceType->kind == TypeKind::INTEGER) {
        return true;  // Allow integer to float conversion
    }
    
    return false;
}

std::shared_ptr<TypeInfo> TypeSystem::getCommonType(std::shared_ptr<TypeInfo> type1, std::shared_ptr<TypeInfo> type2) {
    if (type1->name == type2->name) {
        return type1;
    }
    
    // If one is integer and the other is float, return float
    if ((type1->kind == TypeKind::INTEGER && type2->kind == TypeKind::FLOAT) ||
        (type1->kind == TypeKind::FLOAT && type2->kind == TypeKind::INTEGER)) {
        return getFloatType();
    }
    
    // For now, return unknown if no common type found
    return getUnknownType();
}

std::string TypeSystem::typeToString(std::shared_ptr<TypeInfo> type) {
    return type->name;
}

}
}