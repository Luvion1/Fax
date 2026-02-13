#ifndef SYMBOL_TABLE_HPP
#define SYMBOL_TABLE_HPP

#include "type_system.hpp"
#include <string>
#include <map>
#include <stack>
#include <memory>

namespace fax {
namespace codegen {

struct Symbol {
    std::string name;
    std::shared_ptr<TypeInfo> type;
    bool isMutable;
    int scopeLevel;
    std::string llvmName;  // The corresponding LLVM variable name
    
    Symbol(const std::string& n, std::shared_ptr<TypeInfo> t, bool mut, int level, const std::string& llName)
        : name(n), type(t), isMutable(mut), scopeLevel(level), llvmName(llName) {}
};

class SymbolTable {
private:
    std::stack<std::map<std::string, std::shared_ptr<Symbol>>> scopes;
    int currentScopeLevel = 0;
    int tempVarCounter = 0;
    
public:
    SymbolTable();
    
    // Enter a new scope
    void enterScope();
    
    // Exit current scope
    void exitScope();
    
    // Add a symbol to the current scope
    void addSymbol(const std::string& name, std::shared_ptr<TypeInfo> type, bool isMutable = true);
    
    // Look up a symbol in the current or parent scopes
    std::shared_ptr<Symbol> lookupSymbol(const std::string& name);
    
    // Check if a symbol exists in the current scope only
    bool hasSymbolInCurrentScope(const std::string& name);
    
    // Generate a unique temporary variable name
    std::string generateTempVar();
    
    // Get current scope level
    int getCurrentScopeLevel() const { return currentScopeLevel; }
    
    // Check if table is empty
    bool isEmpty() const { return scopes.empty(); }
};

}
}

#endif