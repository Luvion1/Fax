#ifndef CODEGEN_HPP
#define CODEGEN_HPP

#include <iostream>
#include <string>
#include <vector>
#include <map>
#include <set>
#include "json.hpp"
#include "types.hpp"
#include "optimizer.hpp"

namespace fax {
namespace codegen {

using json = nlohmann::json;

class Codegen {
    json ast;
    int lbl = 0, str = 0;
    std::vector<LoopInfo> loops;
    std::map<std::string, std::string> symbols, returns, pointers, types;
    std::map<std::string, std::map<std::string, int>> structMeta;
    std::map<std::string, std::map<std::string, std::string>> fieldTypes;
    std::map<const json*, int> nodeIds;
    std::map<std::string, std::vector<int>> scopes;
    std::map<std::string, std::string> strings;
    std::set<std::string> globals;

    bool isPointerType(const std::string& type);
    bool isStringType(const std::string& type);
    std::string ensurePointer(Info info, int level);
    std::string mangle(const std::string& name);
    std::string resolve(const std::string& name);
    void indent(int level);
    void emitHeader();
    void defineGlobal(const json& node);
    void process(const json& nodes);
    void emitStatement(const json& node, int level);
    void emitBlock(const json& node, int level);
    Info load(const json& node, int level);
    std::string toRegister(Info info, int level);
    void defineVariable(const json& node, int level);
    void emitAssignment(const json& node, int level);
    void emitIf(const json& node, int level);
    void emitWhile(const json& node, int level);
    void emitFor(const json& node, int level);
    void emitReturn(const json& node, int level);
    void emitCall(const json& node, int level);
    void emitFunction(const json& node);
    void collect(const json& node, std::vector<std::string>& pointers);
    void emitPrint(const json& node, int level);

public:
    Codegen(const json& data);
    void generate();
};

}
}

#endif