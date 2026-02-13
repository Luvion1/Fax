#ifndef OPTIMIZER_HPP
#define OPTIMIZER_HPP

#include "json.hpp"
#include <string>

namespace fax {
namespace codegen {

class Optimizer {
public:
    static nlohmann::json optimize(const nlohmann::json& ast);
    
private:
    static nlohmann::json optimizeConstants(const nlohmann::json& node);
    static nlohmann::json optimizeExpressions(const nlohmann::json& node);
    static nlohmann::json optimizeControlFlow(const nlohmann::json& node);
    static bool isConstant(const nlohmann::json& node, long& value);
};

}
}

#endif