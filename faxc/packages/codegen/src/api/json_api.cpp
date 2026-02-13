#include "json_api.hpp"
#include "../backend/codegen.hpp"
#include <fstream>

namespace fax {
namespace api {

void generate_ir_from_json(const std::string& path) {
    std::ifstream f(path);
    if (!f.good()) {
        std::cerr << "Error: Could not open file " << path << std::endl;
        exit(1);
    }
    
    nlohmann::json ast = nlohmann::json::parse(f);
    fax::codegen::Codegen generator(ast);
    generator.generate();
}

} 
}