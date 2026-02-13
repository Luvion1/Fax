#include <iostream>
#include <fstream>
#include <sstream>
#include "backend/codegen.hpp"
#include "json.hpp"

int main(int argc, char* argv[]) {
    if (argc < 2) {
        std::cerr << "Usage: " << argv[0] << " <input_ast.json>\n";
        return 1;
    }

    std::ifstream inputFile(argv[1]);
    if (!inputFile.is_open()) {
        std::cerr << "Error: Could not open file " << argv[1] << "\n";
        return 1;
    }

    nlohmann::json jsonData;
    try {
        inputFile >> jsonData;
    } catch (const std::exception& e) {
        std::cerr << "Error: Failed to parse JSON - " << e.what() << "\n";
        return 1;
    }

    fax::codegen::Codegen codegen(jsonData);
    codegen.generate();

    return 0;
}