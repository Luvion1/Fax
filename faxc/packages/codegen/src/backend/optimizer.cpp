#include "optimizer.hpp"
#include <iostream>

namespace fax {
namespace codegen {

nlohmann::json Optimizer::optimize(const nlohmann::json& ast) {
    nlohmann::json result = ast;
    
    // Apply optimizations in sequence
    result = optimizeConstants(result);
    result = optimizeExpressions(result);
    result = optimizeControlFlow(result);
    
    return result;
}

bool Optimizer::isConstant(const nlohmann::json& node, long& value) {
    if (node.contains("type") && node["type"] == "NumberLiteral" && node.contains("value")) {
        try {
            std::string val_str = node["value"];
            value = std::stol(val_str);
            return true;
        } catch (...) {
            return false;
        }
    }
    return false;
}

nlohmann::json Optimizer::optimizeConstants(const nlohmann::json& node) {
    if (node.is_object()) {
        nlohmann::json newNode = node;
        
        // Look for binary expressions that can be folded
        if (node.contains("type") && (node["type"] == "BinaryExpression" || node["type"] == "ComparisonExpression")) {
            if (node.contains("left") && node.contains("right") && node.contains("op")) {
                long leftVal, rightVal;
                if (isConstant(node["left"], leftVal) && isConstant(node["right"], rightVal)) {
                    long result;
                    std::string op = node["op"];
                    
                    if (op == "add") result = leftVal + rightVal;
                    else if (op == "sub") result = leftVal - rightVal;
                    else if (op == "mul") result = leftVal * rightVal;
                    else if (op == "sdiv") result = rightVal != 0 ? leftVal / rightVal : 0;
                    else if (op == "eq") result = leftVal == rightVal;
                    else if (op == "ne") result = leftVal != rightVal;
                    else if (op == "slt") result = leftVal < rightVal;
                    else if (op == "sgt") result = leftVal > rightVal;
                    else if (op == "sle") result = leftVal <= rightVal;
                    else if (op == "sge") result = leftVal >= rightVal;
                    else {
                        // Operation not foldable, return original
                        for (auto& [key, val] : node.items()) {
                            if (key != "left" && key != "right") {
                                newNode[key] = val;
                            }
                        }
                        newNode["left"] = optimizeConstants(node["left"]);
                        newNode["right"] = optimizeConstants(node["right"]);
                        return newNode;
                    }
                    
                    // Return constant folded result
                    newNode["type"] = node["type"] == "ComparisonExpression" ? "Boolean" : "NumberLiteral";
                    newNode["value"] = std::to_string(result);
                    if (node.contains("loc")) {
                        newNode["loc"] = node["loc"];
                    }
                    return newNode;
                }
            }
        }
        
        // Recursively optimize children
        for (auto& [key, val] : node.items()) {
            if (val.is_object() || val.is_array()) {
                newNode[key] = optimizeConstants(val);
            }
        }
        
        return newNode;
    } else if (node.is_array()) {
        nlohmann::json newArray = nlohmann::json::array();
        for (const auto& item : node) {
            newArray.push_back(optimizeConstants(item));
        }
        return newArray;
    }
    
    return node;
}

nlohmann::json Optimizer::optimizeExpressions(const nlohmann::json& node) {
    if (node.is_object()) {
        nlohmann::json newNode = node;
        
        // Look for redundant operations
        if (node.contains("type") && node["type"] == "BinaryExpression" && 
            node.contains("left") && node.contains("right") && node.contains("op")) {
            
            std::string op = node["op"];
            
            // Optimize x + 0 = x, x * 1 = x, etc.
            if (node["right"].contains("type") && node["right"]["type"] == "NumberLiteral" && 
                node["right"].contains("value")) {
                try {
                    std::string rval_str = node["right"]["value"];
                    long rval = std::stol(rval_str);
                    
                    if ((op == "add" || op == "sub") && rval == 0) {
                        return optimizeExpressions(node["left"]);
                    } else if ((op == "mul" || op == "sdiv") && rval == 1) {
                        return optimizeExpressions(node["left"]);
                    } else if (op == "mul" && rval == 0) {
                        // x * 0 = 0
                        nlohmann::json zeroNode = {
                            {"type", "NumberLiteral"},
                            {"value", "0"}
                        };
                        if (node.contains("loc")) {
                            zeroNode["loc"] = node["loc"];
                        }
                        return zeroNode;
                    }
                } catch (...) {
                    // Not a valid number, continue with normal processing
                }
            }
        }
        
        // Recursively optimize children
        for (auto& [key, val] : node.items()) {
            if (val.is_object() || val.is_array()) {
                newNode[key] = optimizeExpressions(val);
            }
        }
        
        return newNode;
    } else if (node.is_array()) {
        nlohmann::json newArray = nlohmann::json::array();
        for (const auto& item : node) {
            newArray.push_back(optimizeExpressions(item));
        }
        return newArray;
    }
    
    return node;
}

nlohmann::json Optimizer::optimizeControlFlow(const nlohmann::json& node) {
    if (node.is_object()) {
        nlohmann::json newNode = node;
        
        // Optimize if statements with constant conditions
        if (node.contains("type") && node["type"] == "IfStatement" && node.contains("condition")) {
            if (node["condition"].contains("type") && node["condition"]["type"] == "Boolean" &&
                node["condition"].contains("value")) {
                
                bool condition = node["condition"]["value"] == "true";
                
                if (condition) {
                    // Condition is always true, return then branch
                    if (node.contains("then_branch")) {
                        return optimizeControlFlow(node["then_branch"]);
                    }
                } else {
                    // Condition is always false, return else branch if exists
                    if (node.contains("else_branch")) {
                        return optimizeControlFlow(node["else_branch"]);
                    }
                }
            }
        }
        
        // Recursively optimize children
        for (auto& [key, val] : node.items()) {
            if (val.is_object() || val.is_array()) {
                newNode[key] = optimizeControlFlow(val);
            }
        }
        
        return newNode;
    } else if (node.is_array()) {
        nlohmann::json newArray = nlohmann::json::array();
        for (const auto& item : node) {
            newArray.push_back(optimizeControlFlow(item));
        }
        return newArray;
    }
    
    return node;
}

}
}