#include "analyzer.hpp"
#include <iostream>

namespace fax {
namespace codegen {

Analyzer::AnalysisResult Analyzer::analyze(const nlohmann::json& ast) {
    AnalysisResult result;
    
    if (ast.contains("body") && ast["body"].is_array()) {
        for (const auto& node : ast["body"]) {
            analyzeNode(node, result);
        }
    } else {
        analyzeNode(ast, result);
    }
    
    return result;
}

void Analyzer::analyzeNode(const nlohmann::json& node, AnalysisResult& result, std::string currentFunction) {
    if (!node.is_object() || !node.contains("type")) return;
    
    std::string type = node["type"];
    
    if (type == "FunctionDeclaration") {
        analyzeFunction(node, result);
        currentFunction = node.value("name", "");
    } else if (type == "VariableDeclaration") {
        if (node.contains("name")) {
            std::string varName = node["name"];
            result.variableCounts[currentFunction + "::" + varName]++;
        }
    } else if (type == "Identifier") {
        if (node.contains("value")) {
            std::string varName = node["value"];
            result.variableCounts[currentFunction + "::" + varName]++;
        }
    } else if (type == "CallExpression") {
        if (node.contains("name")) {
            std::string funcName = node["name"];
            result.functionCalls[currentFunction].insert(funcName);
            result.hasSideEffects = true; // Function calls may have side effects
        }
    } else if (type == "Assignment") {
        result.hasSideEffects = true;
    } else if (type == "ReturnStatement") {
        result.hasSideEffects = true;
    } else if (type == "IfStatement" || type == "WhileStatement" || type == "ForStatement") {
        result.hasSideEffects = true;
    }
    
    // Recursively analyze child nodes
    for (auto& [key, value] : node.items()) {
        if (value.is_object()) {
            analyzeNode(value, result, currentFunction);
        } else if (value.is_array()) {
            for (const auto& item : value) {
                if (item.is_object()) {
                    analyzeNode(item, result, currentFunction);
                }
            }
        }
    }
}

void Analyzer::analyzeFunction(const nlohmann::json& node, AnalysisResult& result) {
    if (!node.contains("name")) return;
    
    std::string funcName = node["name"];
    
    // Analyze function body for variable usage
    if (node.contains("body") && node["body"].is_array()) {
        std::set<std::string> localVars;
        
        // First pass: collect local variables
        for (const auto& stmt : node["body"]) {
            if (stmt.contains("type") && stmt["type"] == "VariableDeclaration" && stmt.contains("name")) {
                localVars.insert(stmt["name"]);
            }
        }
        
        // Second pass: analyze usage
        for (const auto& stmt : node["body"]) {
            analyzeNode(stmt, result, funcName);
        }
    }
}

}
}