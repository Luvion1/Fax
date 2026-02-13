#ifndef ANALYZER_HPP
#define ANALYZER_HPP

#include "json.hpp"
#include <string>
#include <map>
#include <set>

namespace fax {
namespace codegen {

class Analyzer {
public:
    struct AnalysisResult {
        std::map<std::string, int> variableCounts;
        std::map<std::string, std::set<std::string>> functionCalls;
        std::set<std::string> unusedVariables;
        std::set<std::string> unusedFunctions;
        bool hasSideEffects = false;
    };
    
    static AnalysisResult analyze(const nlohmann::json& ast);
    
private:
    static void analyzeNode(const nlohmann::json& node, AnalysisResult& result, std::string currentFunction = "");
    static void analyzeFunction(const nlohmann::json& node, AnalysisResult& result);
    static void analyzeVariableUsage(const nlohmann::json& node, AnalysisResult& result, std::set<std::string>& localVars);
};

}
}

#endif