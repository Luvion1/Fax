import sys
import json
import os

def optimize_ast(root_ast):
    """
    Optimasi AST menggunakan pendekatan iteratif (stack) untuk menghindari RecursionError.
    """
    stack = [root_ast]
    while stack:
        ast = stack.pop()
        if isinstance(ast, dict):
            ast["metadata"] = ast.get("metadata", {})
            ast["metadata"]["optimizer"] = "Python v3.11 (Iterative)"
            ast["metadata"]["optimized"] = True
            
            if "body" in ast and isinstance(ast["body"], list):
                for item in ast["body"]:
                    stack.append(item)
    return root_ast

def main():
    if len(sys.argv) < 2:
        print(json.dumps({"error": "No input file provided"}))
        sys.exit(1)

    input_file = sys.argv[1]
    
    try:
        with open(input_file, 'r') as f:
            ast_data = json.load(f)
        
        optimized_ast = optimize_ast(ast_data)
        
        # Output kembali ke TS via stdout
        print(json.dumps(optimized_ast, indent=2))
        
    except Exception as e:
        print(json.dumps({"error": str(e)}))
        sys.exit(1)

if __name__ == "__main__":
    main()
