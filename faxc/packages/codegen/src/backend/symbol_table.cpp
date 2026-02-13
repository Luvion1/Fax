#include "symbol_table.hpp"

namespace fax {
namespace codegen {

SymbolTable::SymbolTable() {
    // Enter the global scope initially
    enterScope();
}

void SymbolTable::enterScope() {
    scopes.emplace();
    currentScopeLevel++;
}

void SymbolTable::exitScope() {
    if (!scopes.empty()) {
        scopes.pop();
        currentScopeLevel--;
    }
}

void SymbolTable::addSymbol(const std::string& name, std::shared_ptr<TypeInfo> type, bool isMutable) {
    if (!scopes.empty()) {
        std::string llvmName = "%" + name + "_" + std::to_string(currentScopeLevel) + "_" + std::to_string(tempVarCounter++);
        scopes.top()[name] = std::make_shared<Symbol>(name, type, isMutable, currentScopeLevel, llvmName);
    }
}

std::shared_ptr<Symbol> SymbolTable::lookupSymbol(const std::string& name) {
    // Search from the current scope up to the global scope
    // Since std::stack doesn't support reverse iteration, we need to iterate differently
    std::vector<std::map<std::string, std::shared_ptr<Symbol>>> temp_scopes;
    
    // Pop all scopes temporarily to access them in reverse order
    while (!scopes.empty()) {
        temp_scopes.push_back(scopes.top());
        scopes.pop();
    }
    
    // Search from most recent scope to oldest
    for (auto it = temp_scopes.rbegin(); it != temp_scopes.rend(); ++it) {
        auto symbolIter = it->find(name);
        if (symbolIter != it->end()) {
            // Restore the scopes
            for (auto rit = temp_scopes.rbegin(); rit != temp_scopes.rend(); ++rit) {
                scopes.push(*rit);
            }
            return symbolIter->second;
        }
    }
    
    // Restore the scopes
    for (auto rit = temp_scopes.rbegin(); rit != temp_scopes.rend(); ++rit) {
        scopes.push(*rit);
    }
    
    return nullptr;
}

bool SymbolTable::hasSymbolInCurrentScope(const std::string& name) {
    if (!scopes.empty()) {
        return scopes.top().find(name) != scopes.top().end();
    }
    return false;
}

std::string SymbolTable::generateTempVar() {
    return "%tmp" + std::to_string(tempVarCounter++);
}

}
}