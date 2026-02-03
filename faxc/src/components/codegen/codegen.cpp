#include <iostream>
#include <fstream>
#include <string>
#include <vector>
#include <map>
#include <algorithm>
#include <sstream>
#include "../../../include/json.hpp"

using json = nlohmann::json;

class Codegen {
    json ast;
    int label_counter = 0;
    int current_root_slot = 0;
    const int MAX_ROOTS = 128;
    
    struct NodeInfo {
        std::string reg;
        std::string type;
    };

    std::map<std::string, std::string> symbol_table; 
    std::map<std::string, std::map<std::string, int>> struct_metadata;
    std::map<std::string, std::vector<std::string>> struct_field_order;
    std::map<std::string, std::map<std::string, std::string>> struct_field_types;
    std::map<std::string, std::string> function_return_types;
    std::map<std::string, std::string> is_ptr_var;
    std::map<std::string, std::string> var_type; 
    std::map<std::string, int> var_to_slot;
    std::map<std::string, int> local_vars; 
    std::map<const json*, int> node_to_id;
    std::map<const json*, std::string> node_mangled_name;
    std::map<std::string, std::vector<int>> scope_stack;
    std::map<std::string, std::string> global_strings;
    int string_counter = 0;
    std::stringstream global_stream;

    std::string get_active_mangled(std::string name) {
        if (scope_stack.count(name) && !scope_stack[name].empty()) {
            return name + "_" + std::to_string(scope_stack[name].back());
        }
        return name + "_ptr"; 
    }

    std::string get_active_ptr(std::string name) {
        if (scope_stack.count(name) && !scope_stack[name].empty()) {
            return "%" + name + "_" + std::to_string(scope_stack[name].back()) + "_ptr";
        }
        return "%" + name + "_ptr";
    }

public:
    Codegen(const std::string& path) {
        srand(time(NULL));
        std::ifstream f(path);
        if (f.good()) {
            ast = json::parse(f);
        } else {
            std::cerr << "Error opening file: " << path << std::endl;
            exit(1);
        }
    }

    int next_label() { return label_counter++; }

    void indent(int level) {
        for (int i = 0; i < level; ++i) std::cout << "    ";
    }

    void emit_header() {
        std::cout << "; Fax Codegen\n";
        std::cout << "target triple = \"x86_64-pc-linux-gnu\"\n\n";
        std::cout << "declare void @fax_fgc_init()\n";
        std::cout << "declare i8* @fax_fgc_alloc(i64, i64*, i64)\n";
        std::cout << "declare void @fax_fgc_collect()\n";
        std::cout << "declare void @fax_fgc_register_root(i8**, i64)\n";
        std::cout << "declare i64 @printf(i8*, ...)\n\n";
        std::cout << "define void @Std_io_collect_fgc() {\nentry:\n    call void @fax_fgc_collect()\n    ret void\n}\n\n";
        std::cout << "@fmt_int = private unnamed_addr constant [5 x i8] c\"%ld\\0A\\00\"\n";
        std::cout << "@fmt_str = private unnamed_addr constant [4 x i8] c\"%s\\0A\\00\"\n";
    }

    void process_nodes(const json& nodes) {
        for (auto& node : nodes) {
            if (node.contains("type") && node["type"] == "StructDeclaration") {
                emit_struct_def(node);
            }
            if (node.contains("type") && node["type"] == "ClassDeclaration") {
                emit_class_def(node);
            }
        }
        for (auto& node : nodes) {
            if (node.contains("type") && node["type"] == "FunctionDeclaration") {
                std::string f_name = node["name"];
                std::string r_type = node.contains("returnType") ? node["returnType"].get<std::string>() : "void";
                function_return_types[f_name] = r_type;
            }
        }
        for (auto& node : nodes) {
            if (node.contains("type") && node["type"] == "FunctionDeclaration") {
                emit_function(node);
            }
        }
    }

    void emit_statement(const json& node, int level) {
        if (!node.contains("type")) return;
        
        std::string type = node["type"];
        if (type == "VariableDeclaration") {
            emit_variable_alloc(node, level);
        } else if (type == "IfStatement") {
            emit_if(node, level);
        } else if (type == "WhileStatement") {
            emit_while(node, level);
        } else if (type == "MatchStatement") {
            emit_match(node, level);
        } else if (type == "Assignment") {
            emit_assignment(node, level);
        } else if (type == "CallExpression") {
            if (node["name"] == "io::print") {
                emit_print(node, level);
            } else {
                emit_call_stmt(node, level);
            }
        } else if (type == "MethodCall") {
            load_node_to_reg(node, level);
        } else if (type == "Block") {
            emit_block(node, level);
        } else if (type == "ReturnStatement") {
            emit_return(node, level);
        }
    }

    void emit_block(const json& node, int level) {
        std::map<std::string, size_t> checkpoint;
        for (auto it = scope_stack.begin(); it != scope_stack.end(); ++it) checkpoint[it->first] = it->second.size();

        const json* body_node = &node;
        if (node.is_object() && node.contains("body")) {
            body_node = &node["body"];
        }
        
        if (body_node->is_array()) {
            for (auto& inner_node : *body_node) {
                emit_statement(inner_node, level);
            }
        }

        for (auto it = scope_stack.begin(); it != scope_stack.end(); ++it) {
            size_t old_size = checkpoint.count(it->first) ? checkpoint[it->first] : 0;
            while (it->second.size() > old_size) it->second.pop_back();
        }
    }

    void emit_struct_def(const json& node) {
        std::string name = node["name"];
        struct_metadata[name] = {};
        struct_field_order[name] = {};
        struct_field_types[name] = {};
        std::cout << "%struct." << name << " = type { ";
        bool first = true;
        int idx = 0;
        for (auto& field : node["fields"]) {
            if (!first) std::cout << ", ";
            std::cout << "i64"; 
            std::string f_name = field["name"].get<std::string>();
            struct_field_order[name].push_back(f_name);
            struct_metadata[name][f_name] = idx++;
            if (field.contains("type")) {
                struct_field_types[name][f_name] = field["type"].get<std::string>();
            } else {
                struct_field_types[name][f_name] = "i64";
            }
            first = false;
        }
        std::cout << " }\n";
    }

    void emit_class_def(const json& node) {
        std::string name = node["name"];
        struct_metadata[name] = {};
        struct_field_order[name] = {};
        struct_field_types[name] = {};
        std::cout << "%struct." << name << " = type { ";
        bool first = true;
        int idx = 0;
        for (auto& field : node["fields"]) {
            if (!first) std::cout << ", ";
            std::cout << "i64"; 
            std::string f_name = field["name"].get<std::string>();
            struct_field_order[name].push_back(f_name);
            struct_metadata[name][f_name] = idx++;
            if (field.contains("type")) {
                struct_field_types[name][f_name] = field["type"].get<std::string>();
            } else {
                struct_field_types[name][f_name] = "i64";
            }
            first = false;
        }
        std::cout << " }\n";
        
        for (auto& method : node["methods"]) {
            emit_method(name, method);
        }
    }

    void emit_method(const std::string& class_name, const json& fn_node) {
        std::string method_name = fn_node["name"];
        std::string mangled_name = class_name + "_" + method_name;
        
        symbol_table.clear();
        is_ptr_var.clear();
        var_type.clear();
        local_vars.clear();
        scope_stack.clear();

        std::cout << "\ndefine i64 @" << mangled_name << "(i8* %self_raw";
        if (fn_node.contains("args")) {
            for (auto& arg : fn_node["args"]) {
                if (arg["name"].get<std::string>() == "self") continue;
                std::cout << ", i64 %arg_" << arg["name"].get<std::string>();
                symbol_table[arg["name"].get<std::string>()] = "arg";
            }
        }
        std::cout << ") {\nentry:\n";
        int level = 1;
        
        indent(level); std::cout << "%self_0_ptr = alloca i8*\n";
        indent(level); std::cout << "store i8* %self_raw, i8** %self_0_ptr\n";
        scope_stack["self"] = {0};
        is_ptr_var["self_0"] = "struct";
        var_type["self_0"] = class_name;

        if (fn_node.contains("body")) collect_allocas(fn_node["body"]);

        if (current_root_slot < MAX_ROOTS) {
            int rs = current_root_slot++;
            var_to_slot["self_0"] = rs;
            indent(level); std::cout << "call void @fax_fgc_register_root(i8* %self_raw, i64 " << rs << ")\n";
        }

        if (fn_node.contains("body")) {
            for (auto& node : fn_node["body"]) emit_statement(node, level);
        }
        indent(level); std::cout << "ret i64 0\n}\n";
    }

    void emit_return(const json& node, int level) {
        if (node.contains("expr") && !node["expr"].is_null()) {
             std::string reg = load_node_to_reg(node["expr"], level);
             indent(level); std::cout << "ret i64 " << reg << "\n";
        } else {
             indent(level); std::cout << "ret i64 0\n";
        }
    }

    std::string get_llvm_func_name(std::string name) {
        if (name.length() > 0 && name[0] == '@') return name;
        std::string res = name;
        size_t pos = 0;
        while ((pos = res.find("::", pos)) != std::string::npos) {
            res.replace(pos, 2, "_");
            pos += 1;
        }
        return "@" + res;
    }

    void emit_call_stmt(const json& node, int level) {
        std::string name = node["name"];
        std::vector<std::string> arg_regs;
        if (node.contains("args")) {
            for (auto& arg : node["args"]) {
                arg_regs.push_back(load_node_to_reg(arg, level));
            }
        }
        int id = next_label();
        if (name == "Std::io::collect_fgc") {
            indent(level); std::cout << "call void @Std_io_collect_fgc()\n";
        } else {
            indent(level); std::cout << "%ign" << id << " = call i64 " << get_llvm_func_name(name) << "(";
            for (size_t i = 0; i < arg_regs.size(); ++i) {
                if (i > 0) std::cout << ", ";
                std::cout << "i64 " << arg_regs[i];
            }
            std::cout << ")\n";
        }
    }

    bool is_struct_type(const std::string& t) {
        return struct_metadata.count(t) || t == "Result";
    }

    NodeInfo load_node_info(const json& node, int level) {
        if (node.is_string()) {
            std::string op = node.get<std::string>();
            bool is_num = !op.empty() && std::all_of(op.begin(), op.end(), ::isdigit);
            if (is_num) return {op, "i64"};
            
            if (symbol_table.count(op) && symbol_table[op] == "arg") {
                int id = next_label();
                indent(level); std::cout << "%atmp" << id << " = add i64 %arg_" << op << ", 0\n";
                return {"%atmp" + std::to_string(id), "i64"};
            }

            std::string mangled = get_active_mangled(op);
            std::string ptr_name = get_active_ptr(op);
            int id = next_label();
            std::string type = "i64";
            if (is_ptr_var.count(mangled)) {
                type = is_ptr_var[mangled];
                if (type == "struct") type = var_type[mangled];
            }

            if (is_ptr_var.count(mangled) && is_ptr_var[mangled] != "") {
                 indent(level); std::cout << "%ptr_raw" << id << " = load i8*, i8** " << ptr_name << "\n";
                 return {"%ptr_raw" + std::to_string(id), type}; 
            }
            indent(level); std::cout << "%tmp" << id << " = load i64, i64* " << ptr_name << "\n";
            return {"%tmp" + std::to_string(id), type};
        }

        std::string type = node["type"];
        if (type == "Atomic") {
            return load_node_info(node["value"], level);
        } else if (type == "Assignment") {
            return {emit_assignment(node, level), "i64"};
        } else if (type == "MemberAccess") {
            NodeInfo base = load_node_info(node["base"], level);
            std::string field = node["field"];
            std::string s_type = base.type;
            
            if (!struct_metadata.count(s_type)) {
                return {"0", "i64"};
            }

            int field_idx = struct_metadata[s_type][field];
            std::string f_type = struct_field_types[s_type][field];
            
            int id = next_label();
            indent(level); std::cout << "%cast" << id << " = bitcast i8* " << base.reg << " to %struct." << s_type << "*\n";
            indent(level); std::cout << "%gep" << id << " = getelementptr %struct." << s_type << ", %struct." << s_type << "* %cast" << id << ", i32 0, i32 " << field_idx << "\n";
            
            if (is_struct_type(f_type) || f_type == "array" || f_type == "string") {
                indent(level); std::cout << "%val_ptr" << id << " = load i64, i64* %gep" << id << "\n";
                indent(level); std::cout << "%val" << id << " = inttoptr i64 %val_ptr" << id << " to i8*\n";
                return {"%val" + std::to_string(id), f_type};
            } else {
                indent(level); std::cout << "%val" << id << " = load i64, i64* %gep" << id << "\n";
                return {"%val" + std::to_string(id), f_type};
            }
        } else if (type == "StringLiteral") {
            std::string val = node["value"];
            if (global_strings.find(val) == global_strings.end()) {
                std::string name = "str" + std::to_string(string_counter++);
                global_strings[val] = name;
            }
            std::string g_name = global_strings[val];
            int id = next_label();
            indent(level); std::cout << "%ptr" << id << " = getelementptr inbounds [" << (val.length() + 1) << " x i8], [" << (val.length() + 1) << " x i8]* @" << g_name << ", i32 0, i32 0\n";
            return {"%ptr" + std::to_string(id), "string"};
        } else if (type == "BooleanLiteral") {
            return {(node["value"] == "true") ? "1" : "0", "bool"};
        } else if (type == "BinaryExpression") {
            std::string l = load_node_to_reg(node["left"], level);
            std::string r = load_node_to_reg(node["right"], level);
            std::string op = node["op"];
            int id = next_label();
            indent(level); std::cout << "%bin" << id << " = " << op << " i64 " << l << ", " << r << "\n";
            return {"%bin" + std::to_string(id), "i64"};
        } else if (type == "ComparisonExpression") {
            std::string l = load_node_to_reg(node["left"], level);
            std::string r = load_node_to_reg(node["right"], level);
            std::string op = node["op"];
            int id = next_label();
            indent(level); std::cout << "%cmp" << id << " = icmp " << op << " i64 " << l << ", " << r << "\n";
            indent(level); std::cout << "%zext" << id << " = zext i1 %cmp" << id << " to i64\n";
            return {"%zext" + std::to_string(id), "bool"};
        } else if (type == "LogicalExpression") {
            std::string op = node["op"];
            int id = next_label();
            std::string lhs_reg = load_node_to_reg(node["left"], level);
            std::string res_ptr = "%log_res" + std::to_string(id);
            indent(level); std::cout << res_ptr << " = alloca i64\n";
            indent(level); std::cout << "store i64 " << lhs_reg << ", i64* " << res_ptr << "\n";
            std::string next_lab = "log_next" + std::to_string(id);
            std::string end_lab = "log_end" + std::to_string(id);
            indent(level); std::cout << "%lcond" << id << " = icmp ne i64 " << lhs_reg << ", 0\n";
            if (op == "or") {
                indent(level); std::cout << "br i1 %lcond" << id << ", label %" << end_lab << ", label %" << next_lab << "\n\n";
            } else {
                indent(level); std::cout << "br i1 %lcond" << id << ", label %" << next_lab << ", label %" << end_lab << "\n\n";
            }
            std::cout << next_lab << ":\n";
            std::string rhs_reg = load_node_to_reg(node["right"], level + 1);
            indent(level + 1); std::cout << "store i64 " << rhs_reg << ", i64* " << res_ptr << "\n";
            indent(level + 1); std::cout << "br label %" << end_lab << "\n\n";
            std::cout << end_lab << ":\n";
            int id2 = next_label();
            indent(level); std::cout << "%res" << id2 << " = load i64, i64* " << res_ptr << "\n";
            return {"%res" + std::to_string(id2), "bool"};
        } else if (type == "TryExpression") {
            NodeInfo info = load_node_info(node["expr"], level);
            int id = next_label();
            indent(level); std::cout << "%res_ptr" << id << " = bitcast i8* " << info.reg << " to %struct.Result*\n";
            indent(level); std::cout << "%tag_ptr" << id << " = getelementptr %struct.Result, %struct.Result* %res_ptr" << id << ", i32 0, i32 0\n";
            indent(level); std::cout << "%tag" << id << " = load i64, i64* %tag_ptr" << id << "\n";
            indent(level); std::cout << "%is_err" << id << " = icmp ne i64 %tag" << id << ", 0\n";
            std::string err_lab = "try_err" + std::to_string(id);
            std::string ok_lab = "try_ok" + std::to_string(id);
            indent(level); std::cout << "br i1 %is_err" << id << ", label %" << err_lab << ", label %" << ok_lab << "\n\n";
            std::cout << err_lab << ":\n";
            indent(level + 1); std::cout << "%err_res_i64" << id << " = ptrtoint i8* " << info.reg << " to i64\n";
            indent(level + 1); std::cout << "ret i64 %err_res_i64\n\n";
            std::cout << ok_lab << ":\n";
            indent(level + 1); std::cout << "%val_ptr" << id << " = getelementptr %struct.Result, %struct.Result* %res_ptr" << id << ", i32 0, i32 1\n";
            indent(level + 1); std::cout << "%val" << id << " = load i64, i64* %val_ptr" << id << "\n";
            return {"%val" + std::to_string(id), "i64"};
        } else if (type == "UnaryExpression") {
            std::string op = node["op"];
            int id = next_label();
            if (op == "-") {
                indent(level); std::cout << "%un" << id << " = sub i64 0, " << load_node_to_reg(node["right"], level) << "\n";
                return {"%un" + std::to_string(id), "i64"};
            } else if (op == "!") {
                indent(level); std::cout << "%un" << id << " = xor i64 " << load_node_to_reg(node["right"], level) << ", 1\n";
                return {"%un" + std::to_string(id), "bool"};
            }
        } else if (type == "IndexAccess") {
            NodeInfo base = load_node_info(node["base"], level);
            std::string idx_reg = load_node_to_reg(node["index"], level);
            int id = next_label();
            indent(level); std::cout << "%arr_ptr" << id << " = bitcast i8* " << base.reg << " to i64*\n";
            indent(level); std::cout << "%gep" << id << " = getelementptr i64, i64* %arr_ptr" << id << ", i64 " << idx_reg << "\n";
            indent(level); std::cout << "%val" << id << " = load i64, i64* %gep" << id << "\n";
            return {"%val" + std::to_string(id), "i64"};
        } else if (type == "CallExpression") {
             std::string name = node["name"];
             if (name == "Ok" || name == "Err") {
                 std::string val_reg = load_node_to_reg(node["args"][0], level);
                 int id = next_label();
                 indent(level); std::cout << "%res_raw" << id << " = call i8* @fax_fgc_alloc(i64 24, i64* null, i64 0)\n";
                 indent(level); std::cout << "%res_cast" << id << " = bitcast i8* %res_raw" << id << " to %struct.Result*\n";
                 indent(level); std::cout << "%tag_ptr" << id << " = getelementptr %struct.Result, %struct.Result* %res_cast" << id << ", i32 0, i32 0\n";
                 indent(level); std::cout << "store i64 " << (name == "Ok" ? "0" : "1") << ", i64* %tag_ptr" << id << "\n";
                 int val_idx = (name == "Ok" ? 1 : 2);
                 indent(level); std::cout << "%val_ptr" << id << " = getelementptr %struct.Result, %struct.Result* %res_cast" << id << ", i32 0, i32 " << val_idx << "\n";
                 indent(level); std::cout << "store i64 " << val_reg << ", i64* %val_ptr" << id << "\n";
                 return {"%res_raw" + std::to_string(id), "Result"};
             }
             std::vector<std::string> arg_regs;
             if (node.contains("args")) for (auto& arg : node["args"]) arg_regs.push_back(load_node_to_reg(arg, level));
             int id = next_label();
             indent(level); std::cout << "%res" << id << " = call i64 " << get_llvm_func_name(name) << "(";
             for (size_t i = 0; i < arg_regs.size(); ++i) {
                 if (i > 0) std::cout << ", ";
                 std::cout << "i64 " << arg_regs[i];
             }
             std::cout << ")\n";
             std::string r_type = function_return_types.count(name) ? function_return_types[name] : "i64";
             if (is_struct_type(r_type) || r_type == "array" || r_type == "string") {
                 int cid = next_label();
                 indent(level); std::cout << "%call_ptr" << cid << " = inttoptr i64 %res" << id << " to i8*\n";
                 return {"%call_ptr" + std::to_string(cid), r_type};
             }
             return {"%res" + std::to_string(id), r_type};
        } else if (type == "MethodCall") {
             NodeInfo base = load_node_info(node["base"], level);
             std::string s_type = base.type;
             std::string method = node["method"];
             std::vector<std::string> arg_regs;
             if (node.contains("args")) for (auto& arg : node["args"]) arg_regs.push_back(load_node_to_reg(arg, level));
             int id = next_label();
             indent(level); std::cout << "%mres" << id << " = call i64 @" << s_type << "_" << method << "(i8* " << base.reg;
             for (auto& ar : arg_regs) std::cout << ", i64 " << ar;
             std::cout << ")\n";
             return {"%mres" + std::to_string(id), "i64"}; 
        }
        return {"0", "i64"};
    }

    std::string load_node_to_reg(const json& node, int level) {
        NodeInfo info = load_node_info(node, level);
        if (info.reg.length() > 0 && info.reg[0] == '%') {
             if (is_struct_type(info.type) || info.type == "array" || info.type == "string") {
                 int id = next_label();
                 indent(level); std::cout << "%cast_i64" << id << " = ptrtoint i8* " << info.reg << " to i64\n";
                 return "%cast_i64" + std::to_string(id);
             }
        }
        return info.reg;
    }

    void emit_if(const json& node, int level) {
        std::string cond_reg = load_node_to_reg(node["condition"], level);
        int id = next_label();
        indent(level); std::cout << "%cond_bit" << id << " = icmp ne i64 " << cond_reg << ", 0\n";
        bool has_else = node.contains("else_branch") && node["else_branch"].size() > 0;
        std::string f_lab = has_else ? "else" : "end";
        indent(level); std::cout << "br i1 %cond_bit" << id << ", label %then" << id << ", label %" << f_lab << id << "\n\n";
        std::cout << "then" << id << ":\n";
        std::map<std::string, size_t> checkpoint;
        for (auto it = scope_stack.begin(); it != scope_stack.end(); ++it) checkpoint[it->first] = it->second.size();
        for (auto& inner : node["then_branch"]) emit_statement(inner, level + 1);
        for (auto it = scope_stack.begin(); it != scope_stack.end(); ++it) {
            size_t old_size = checkpoint.count(it->first) ? checkpoint[it->first] : 0;
            while (it->second.size() > old_size) it->second.pop_back();
        }
        indent(level + 1); std::cout << "br label %end" << id << "\n\n";
        if (has_else) {
            std::cout << "else" << id << ":\n";
            std::map<std::string, size_t> e_checkpoint;
            for (auto it = scope_stack.begin(); it != scope_stack.end(); ++it) e_checkpoint[it->first] = it->second.size();
            for (auto& inner : node["else_branch"]) emit_statement(inner, level + 1);
            for (auto it = scope_stack.begin(); it != scope_stack.end(); ++it) {
                size_t old_size = e_checkpoint.count(it->first) ? e_checkpoint[it->first] : 0;
                while (it->second.size() > old_size) it->second.pop_back();
            }
            indent(level + 1); std::cout << "br label %end" << id << "\n\n";
        }
        std::cout << "end" << id << ":\n";
    }

    void emit_match(const json& node, int level) {
        std::string expr_reg = load_node_to_reg(node["expr"], level);
        int match_id = next_label();
        std::string end_label = "match_end" + std::to_string(match_id);
        int arm_idx = 0;
        for (auto& arm : node["arms"]) {
            std::string val_reg = load_node_to_reg(arm["value"], level);
            int id = next_label();
            indent(level); std::cout << "%mcmp" << id << " = icmp eq i64 " << expr_reg << ", " << val_reg << "\n";
            std::string next_arm_label = "match" + std::to_string(match_id) + "_arm" + std::to_string(arm_idx + 1);
            if (arm_idx == (int)node["arms"].size() - 1 && node["default"].is_null()) next_arm_label = end_label;
            else if (arm_idx == (int)node["arms"].size() - 1 && !node["default"].is_null()) next_arm_label = "match" + std::to_string(match_id) + "_default";
            indent(level); std::cout << "br i1 %mcmp" << id << ", label %match" << std::to_string(match_id) << "_body" << arm_idx << ", label %" << next_arm_label << "\n\n";
            std::cout << "match" << std::to_string(match_id) << "_body" << arm_idx << ":\n";
            emit_block(arm["body"], level + 1);
            indent(level + 1); std::cout << "br label %" << end_label << "\n\n";
            std::cout << next_arm_label << ":\n";
            arm_idx++;
        }
        if (!node["default"].is_null()) {
            emit_block(node["default"], level + 1);
            indent(level + 1); std::cout << "br label %" << end_label << "\n\n";
            std::cout << end_label << ":\n";
        } else if (node["arms"].empty()) std::cout << end_label << ":\n";
    }

    std::string emit_assignment(const json& node, int level) {
        std::string val_reg = load_node_to_reg(node["expr"], level);
        const json& target = node["target"];
        if (target.is_object() && target.contains("type") && target["type"] == "MemberAccess") {
            NodeInfo base = load_node_info(target["base"], level);
            std::string field = target["field"];
            std::string s_type = base.type;
            int field_idx = struct_metadata[s_type][field];
            int id = next_label();
            indent(level); std::cout << "%gep" << id << " = getelementptr %struct." << s_type << ", %struct." << s_type << "* " << (is_struct_type(base.type) ? "" : "(bitcast i8* ") << base.reg << (is_struct_type(base.type) ? "" : " to %struct." + s_type + "*)") << ", i32 0, i32 " << field_idx << "\n";
            indent(level); std::cout << "store i64 " << val_reg << ", i64* %gep" << id << "\n";
        } else if (target.is_object() && target.contains("type") && target["type"] == "IndexAccess") {
            NodeInfo base = load_node_info(target["base"], level);
            std::string idx_reg = load_node_to_reg(target["index"], level);
            int id = next_label();
            indent(level); std::cout << "%arr_ptr" << id << " = bitcast i8* " << base.reg << " to i64*\n";
            indent(level); std::cout << "%gep" << id << " = getelementptr i64, i64* %arr_ptr" << id << ", i64 " << idx_reg << "\n";
            indent(level); std::cout << "store i64 " << val_reg << ", i64* %gep" << id << "\n";
        } else {
            std::string name = target.is_string() ? target.get<std::string>() : target["value"].get<std::string>();
            std::string mangled = get_active_mangled(name);
            std::string ptr_name = get_active_ptr(name);
            if (is_ptr_var.count(mangled) && is_ptr_var[mangled] != "") {
                 int id = next_label();
                 indent(level); std::cout << "%cast_val" << id << " = inttoptr i64 " << val_reg << " to i8*\n";
                 indent(level); std::cout << "store i8* %cast_val" << id << ", i8** " << ptr_name << "\n";
                 if (var_to_slot.count(mangled)) {
                     indent(level); std::cout << "call void @fax_fgc_register_root(i8* %cast_val, i64 " << var_to_slot[mangled] << ")\n";
                 }
            } else indent(level); std::cout << "store i64 " << val_reg << ", i64* " << ptr_name << "\n";
        }
        return val_reg;
    }

    void emit_while(const json& node, int level) {
        int id = next_label();
        indent(level); std::cout << "br label %cond" << id << "\n\n";
        std::cout << "cond" << id << ":\n";
        std::string cond_reg = load_node_to_reg(node["condition"], level + 1);
        indent(level + 1); std::cout << "%cmp" << id << " = icmp ne i64 " << cond_reg << ", 0\n";
        indent(level + 1); std::cout << "br i1 %cmp" << id << ", label %body" << id << ", label %end" << id << "\n\n";
        std::cout << "body" << id << ":\n";
        std::map<std::string, size_t> checkpoint;
        for (auto it = scope_stack.begin(); it != scope_stack.end(); ++it) checkpoint[it->first] = it->second.size();
        for (auto& inner : node["body"]) emit_statement(inner, level + 1);
        for (auto it = scope_stack.begin(); it != scope_stack.end(); ++it) {
            size_t old_size = checkpoint.count(it->first) ? checkpoint[it->first] : 0;
            while (it->second.size() > old_size) it->second.pop_back();
        }
        indent(level + 1); std::cout << "br label %cond" << id << "\n\n";
        std::cout << "end" << id << ":\n";
    }

    void collect_allocas(const json& node) {
        if (node.is_array()) { for (auto& item : node) collect_allocas(item); return; }
        if (!node.is_object()) return;
        if (node.contains("type") && node["type"] == "VariableDeclaration") {
            std::string var_name = node["name"];
            int id = next_label();
            local_vars[var_name] = id;
            node_to_id[&node] = id;
            std::string mangled = var_name + "_" + std::to_string(id);
            node_mangled_name[&node] = mangled;
            std::string ptr_name = "%" + mangled + "_ptr";
            
            std::string vtype = "i64";
            if (node.contains("structInit")) vtype = node["structInit"]["name"];
            else if (node.contains("expr")) {
                std::string etype = node["expr"]["type"];
                if (etype == "StructLiteral") vtype = node["expr"]["name"];
                else if (etype == "ArrayLiteral") vtype = "array";
                else if (etype == "StringLiteral") vtype = "string";
                else if (etype == "CallExpression") {
                    std::string fname = node["expr"]["name"];
                    vtype = function_return_types.count(fname) ? function_return_types[fname] : "i64";
                }
            }
            
            bool is_ptr = is_struct_type(vtype) || vtype == "array" || vtype == "string" || vtype == "Result";
            if (is_ptr) {
                is_ptr_var[mangled] = (vtype == "array" || vtype == "string") ? vtype : "struct";
                var_type[mangled] = vtype;
                if (current_root_slot < MAX_ROOTS) {
                    var_to_slot[mangled] = current_root_slot++;
                }
            }
            
            indent(1); std::cout << ptr_name << " = alloca " << (is_ptr ? "i8*" : "i64") << "\n";
            if (is_ptr) {
                indent(1); std::cout << "store i8* null, i8** " << ptr_name << "\n";
            } else {
                indent(1); std::cout << "store i64 0, i64* " << ptr_name << "\n";
            }
        }
        if (node.contains("body")) collect_allocas(node["body"]);
        if (node.contains("then_branch")) collect_allocas(node["then_branch"]);
        if (node.contains("else_branch")) collect_allocas(node["else_branch"]);
        if (node.contains("arms")) for (auto& arm : node["arms"]) collect_allocas(arm["body"]);
        if (node.contains("default") && !node["default"].is_null()) collect_allocas(node["default"]);
        if (node.contains("methods")) for (auto& m : node["methods"]) collect_allocas(m["body"]);
    }

    void emit_variable_alloc(const json& node, int level) {
        std::string var_name = node["name"];
        int id = node_to_id[&node];
        scope_stack[var_name].push_back(id);
        std::string mangled = node_mangled_name[&node];
        std::string ptr_name = "%" + mangled + "_ptr";
        
        if (node.contains("structInit") || (node.contains("expr") && node["expr"]["type"] == "StructLiteral")) {
            const json& init_node = node.contains("structInit") ? node["structInit"] : node["expr"];
            std::string s_name = init_node["name"];
            int fields_count = struct_metadata[s_name].size();
            std::vector<int> ptr_offsets;
            for (auto const& f_name : struct_field_order[s_name]) {
                int f_idx = struct_metadata[s_name][f_name];
                std::string f_type = struct_field_types[s_name][f_name];
                if (is_struct_type(f_type) || f_type == "array" || f_type == "string") ptr_offsets.push_back(f_idx * 8);
            }
            int alloc_id = next_label();
            
            if (!ptr_offsets.empty()) {
                std::string map_name = "pmap_" + std::to_string(alloc_id);
                global_stream << "@" << map_name << " = private unnamed_addr constant [" << ptr_offsets.size() << " x i64] [";
                for (size_t i = 0; i < ptr_offsets.size(); ++i) { if (i > 0) global_stream << ", "; global_stream << "i64 " << ptr_offsets[i]; }
                global_stream << "]\n";
                indent(level); std::cout << "%map_ptr" << alloc_id << " = getelementptr [" << ptr_offsets.size() << " x i64], [" << ptr_offsets.size() << " x i64]* @" << map_name << ", i32 0, i32 0\n";
                indent(level); std::cout << "%" << mangled << "_ptr_raw = call i8* @fax_fgc_alloc(i64 " << (fields_count * 8) << ", i64* %map_ptr" << alloc_id << ", i64 " << ptr_offsets.size() << ")\n";
            } else { 
                indent(level); std::cout << "%" << mangled << "_ptr_raw = call i8* @fax_fgc_alloc(i64 " << (fields_count * 8) << ", i64* null, i64 0)\n"; 
            }
            indent(level); std::cout << "store i8* %" << mangled << "_ptr_raw, i8** " << ptr_name << "\n";
            if (var_to_slot.count(mangled)) {
                indent(level); std::cout << "call void @fax_fgc_register_root(i8** " << ptr_name << ", i64 " << var_to_slot[mangled] << ")\n";
            }
            
            indent(level); std::cout << "%" << mangled << "_cast = bitcast i8* %" << mangled << "_ptr_raw to %struct." << s_name << "*\n";
            for (auto& f : init_node["fields"]) {
                std::string f_name = f["name"].get<std::string>();
                int idx = struct_metadata[s_name][f_name];
                NodeInfo f_info = load_node_info(f["expr"], level);
                std::string reg = f_info.reg;
                if (is_struct_type(f_info.type) || f_info.type == "array" || f_info.type == "string") {
                    int cid = next_label();
                    indent(level); std::cout << "%field_cast" << cid << " = ptrtoint i8* " << reg << " to i64\n";
                    reg = "%field_cast" + std::to_string(cid);
                }
                int fid = next_label();
                indent(level); std::cout << "%gep" << fid << " = getelementptr %struct." << s_name << ", %struct." << s_name << "* %" << mangled << "_cast, i32 0, i32 " << idx << "\n";
                indent(level); std::cout << "store i64 " << reg << ", i64* %gep" << fid << "\n";
            }
        } else if (node.contains("expr")) {
            NodeInfo info = load_node_info(node["expr"], level);
            bool is_ptr = is_ptr_var.count(mangled) && is_ptr_var[mangled] != "";
            std::string reg = info.reg;
            if (is_ptr && reg[0] == '%') {
                int cid = next_label();
                indent(level); std::cout << "%var_cast" << cid << " = ptrtoint i8* " << reg << " to i64\n";
                reg = "%var_cast" + std::to_string(cid);
            }
            if (is_ptr) {
                int cid = next_label();
                indent(level); std::cout << "%ptr_val" << cid << " = inttoptr i64 " << reg << " to i8*\n";
                indent(level); std::cout << "store i8* %ptr_val" << cid << ", i8** " << ptr_name << "\n";
                if (var_to_slot.count(mangled)) {
                    indent(level); std::cout << "call void @fax_fgc_register_root(i8** " << ptr_name << ", i64 " << var_to_slot[mangled] << ")\n";
                }
            } else {
                indent(level); std::cout << "store i64 " << reg << ", i64* " << ptr_name << "\n";
            }
        } else { indent(level); std::cout << "store i64 0, i64* " << ptr_name << "\n"; }
    }

    void emit_function(const json& fn_node) {
        std::string name = fn_node["name"];
        bool is_extern = fn_node.contains("isExtern") && fn_node["isExtern"].get<bool>();
        if (is_extern) {
            std::cout << "\ndeclare i64 @" << name << "(";
            if (fn_node.contains("args")) { bool first = true; for (auto& arg : fn_node["args"]) { if (!first) std::cout << ", "; std::cout << "i64"; first = false; } }
            std::cout << ")\n"; return;
        }
        symbol_table.clear(); is_ptr_var.clear(); var_type.clear(); local_vars.clear(); scope_stack.clear(); 
        std::cout << "\ndefine i64 @" << name << "(";
        if (fn_node.contains("args")) { bool first = true; for (auto& arg : fn_node["args"]) { if (!first) std::cout << ", "; std::cout << "i64 %arg_" << arg["name"].get<std::string>(); symbol_table[arg["name"].get<std::string>()] = "arg"; first = false; } }
        std::cout << ") {\nentry:\n";
        if (name == "main") {
            indent(1); std::cout << "call void @fax_fgc_init()\n";
        }
        if (fn_node.contains("body")) collect_allocas(fn_node["body"]);
        if (fn_node.contains("args")) {
            for (auto& arg : fn_node["args"]) {
                std::string arg_name = arg["name"].get<std::string>();
                std::string arg_type = arg["type"].get<std::string>();
                if (is_struct_type(arg_type) || arg_type == "array" || arg_type == "string") {
                    if (current_root_slot < MAX_ROOTS) {
                        int rs = current_root_slot++;
                        std::string arg_ptr = "%" + arg_name + "_arg_ptr";
                        indent(1); std::cout << arg_ptr << " = alloca i8*\n";
                        indent(1); std::cout << "%arg_tmp" << rs << " = inttoptr i64 %arg_" << arg_name << " to i8*\n";
                        indent(1); std::cout << "store i8* %arg_tmp" << rs << ", i8** " << arg_ptr << "\n";
                        indent(1); std::cout << "call void @fax_fgc_register_root(i8** " << arg_ptr << ", i64 " << rs << ")\n";
                    }
                }
            }
        }
        if (fn_node.contains("body")) for (auto& node : fn_node["body"]) emit_statement(node, 1);
        indent(1); std::cout << "ret i64 0\n}\n";
    }

    void emit_print(const json& node, int level) {
        NodeInfo info = load_node_info(node["expr"], level);
        int id = next_label();
        if (info.type == "string") {
            indent(level); std::cout << "%ign" << id << " = call i64 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @fmt_str, i32 0, i32 0), i8* " << info.reg << ")\n";
        } else {
            std::string reg = info.reg;
            if (reg.length() > 0 && reg[0] == '%' && (is_struct_type(info.type) || info.type == "array" || info.type == "string")) {
                int cid = next_label();
                indent(level); std::cout << "%cast_i64" << cid << " = ptrtoint i8* " << reg << " to i64\n";
                reg = "%cast_i64" + std::to_string(cid);
            }
            indent(level); std::cout << "%ign" << id << " = call i64 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @fmt_int, i32 0, i32 0), i64 " << reg << ")\n";
        }
    }

    void emit_globals() {
        std::cout << global_stream.str();
        for (auto const& [val, name] : global_strings) {
            std::string escaped = "";
            for (char c : val) {
                if (c == '\n') escaped += "\\0A"; else if (c == '\t') escaped += "\\09";
                else if (c == '\"') escaped += "\\22"; else if (c == '\\') escaped += "\\5C";
                else escaped += c;
            }
            std::cout << "@" << name << " = private unnamed_addr constant [" << (val.length() + 1) << " x i8] c\"" << escaped << "\\00\"\n";
        }
    }

    void generate() {
        emit_header();
        if (ast.contains("body")) process_nodes(ast["body"]);
        emit_globals();
    }
};

int main(int argc, char* argv[]) {
    if (argc < 2) return 1;
    try { Codegen cg(argv[1]); cg.generate(); } catch (...) { return 1; }
    return 0;
}