#ifndef CODEGEN_TYPES_HPP
#define CODEGEN_TYPES_HPP

#include <string>

namespace fax {
namespace codegen {

struct Info {
    std::string reg;
    std::string type;
};

struct LoopInfo {
    int start;
    int end;
};

} // namespace codegen
} // namespace fax

#endif // CODEGEN_TYPES_HPP
