#ifndef INSTRUCTION_BUILDER_HPP
#define INSTRUCTION_BUILDER_HPP

#include <string>
#include <vector>
#include <iostream>

namespace fax {
namespace codegen {

class InstructionBuilder {
private:
    int labelCounter = 0;
    int tempCounter = 0;
    
public:
    std::string newLabel() {
        return "L" + std::to_string(labelCounter++);
    }
    
    std::string newTemp() {
        return "%t" + std::to_string(tempCounter++);
    }
    
    // Basic arithmetic operations
    std::string add(std::string lhs, std::string rhs, int level = 0) {
        std::string result = newTemp();
        indent(level);
        std::cout << result << " = add i64 " << lhs << ", " << rhs << "\n";
        return result;
    }
    
    std::string sub(std::string lhs, std::string rhs, int level = 0) {
        std::string result = newTemp();
        indent(level);
        std::cout << result << " = sub i64 " << lhs << ", " << rhs << "\n";
        return result;
    }
    
    std::string mul(std::string lhs, std::string rhs, int level = 0) {
        std::string result = newTemp();
        indent(level);
        std::cout << result << " = mul i64 " << lhs << ", " << rhs << "\n";
        return result;
    }
    
    std::string div(std::string lhs, std::string rhs, int level = 0) {
        std::string result = newTemp();
        indent(level);
        std::cout << result << " = sdiv i64 " << lhs << ", " << rhs << "\n";
        return result;
    }
    
    // Comparison operations
    std::string eq(std::string lhs, std::string rhs, int level = 0) {
        std::string result = newTemp();
        indent(level);
        std::cout << result << " = icmp eq i64 " << lhs << ", " << rhs << "\n";
        return result;
    }
    
    std::string ne(std::string lhs, std::string rhs, int level = 0) {
        std::string result = newTemp();
        indent(level);
        std::cout << result << " = icmp ne i64 " << lhs << ", " << rhs << "\n";
        return result;
    }
    
    std::string lt(std::string lhs, std::string rhs, int level = 0) {
        std::string result = newTemp();
        indent(level);
        std::cout << result << " = icmp slt i64 " << lhs << ", " << rhs << "\n";
        return result;
    }
    
    std::string gt(std::string lhs, std::string rhs, int level = 0) {
        std::string result = newTemp();
        indent(level);
        std::cout << result << " = icmp sgt i64 " << lhs << ", " << rhs << "\n";
        return result;
    }
    
    // Branching
    void br(std::string label, int level = 0) {
        indent(level);
        std::cout << "br label %" << label << "\n";
    }
    
    void br_cond(std::string condition, std::string true_label, std::string false_label, int level = 0) {
        indent(level);
        std::cout << "br i1 " << condition << ", label %" << true_label << ", label %" << false_label << "\n";
    }
    
    // Function calls
    std::string call(std::string func_name, std::vector<std::string> args, int level = 0) {
        std::string result = newTemp();
        indent(level);
        std::cout << result << " = call i64 @" << func_name << "(";
        for (size_t i = 0; i < args.size(); ++i) {
            if (i > 0) std::cout << ", ";
            std::cout << "i64 " << args[i];
        }
        std::cout << ")\n";
        return result;
    }
    
    // Memory operations
    std::string alloca(int level = 0) {
        std::string result = newTemp();
        indent(level);
        std::cout << result << " = alloca i64\n";
        return result;
    }
    
    std::string load(std::string ptr, int level = 0) {
        std::string result = newTemp();
        indent(level);
        std::cout << result << " = load i64, i64* " << ptr << "\n";
        return result;
    }
    
    void store(std::string value, std::string ptr, int level = 0) {
        indent(level);
        std::cout << "store i64 " << value << ", i64* " << ptr << "\n";
    }
    
    // Conversion operations
    std::string ptr_to_int(std::string ptr, int level = 0) {
        std::string result = newTemp();
        indent(level);
        std::cout << result << " = ptrtoint i8* " << ptr << " to i64\n";
        return result;
    }
    
    std::string int_to_ptr(std::string value, int level = 0) {
        std::string result = newTemp();
        indent(level);
        std::cout << result << " = inttoptr i64 " << value << " to i8*\n";
        return result;
    }
    
    // Return statement
    void ret(std::string value, int level = 0) {
        indent(level);
        std::cout << "ret i64 " << value << "\n";
    }
    
    // Helper function for indentation
    void indent(int level) {
        for (int i = 0; i < level; ++i) std::cout << "    ";
    }
};

}
}

#endif