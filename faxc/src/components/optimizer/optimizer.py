#!/usr/bin/env python3

"""
FAX Optimizer - AST optimization stage of the compiler

This module implements AST transformations and optimizations for the FAX compiler.
Currently performs metadata annotation; can be extended with graph-based optimizations.
"""

import sys
import json
import os


def optimize_ast(root_ast):
    """
    Optimize AST using an iterative stack-based approach to avoid RecursionError.
    
    This function annotates AST nodes with optimizer metadata and can be extended
    to perform graph-based transformations, constant folding, dead code elimination, etc.
    
    Args:
        root_ast: The AST to optimize (dict/list)
    
    Returns:
        The optimized AST with metadata annotations
    """
    stack = [root_ast]
    while stack:
        ast_node = stack.pop()
        if isinstance(ast_node, dict):
            # Annotate with optimizer metadata
            if "metadata" not in ast_node:
                ast_node["metadata"] = {}
            ast_node["metadata"]["optimizer"] = "Python v3.10+ (Iterative)"
            ast_node["metadata"]["optimized"] = True
            
            # Queue child nodes for processing
            if "body" in ast_node and isinstance(ast_node["body"], list):
                for child_item in ast_node["body"]:
                    stack.append(child_item)
    
    return root_ast


def main():
    """Main entry point for the optimizer."""
    if len(sys.argv) < 2:
        error_output = {
            "error": "No input file provided",
            "code": "E0000"
        }
        print(json.dumps(error_output))
        sys.exit(1)

    input_file = sys.argv[1]
    
    try:
        # Read AST from file
        with open(input_file, 'r') as f:
            ast_data = json.load(f)
        
        # Perform optimization
        optimized_ast = optimize_ast(ast_data)
        
        # Output optimized AST to stdout
        print(json.dumps(optimized_ast, indent=2))
        
    except FileNotFoundError:
        error_output = {
            "error": f"Input file not found: {input_file}",
            "code": "E0001"
        }
        print(json.dumps(error_output))
        sys.exit(1)
    except json.JSONDecodeError as e:
        error_output = {
            "error": f"Failed to parse JSON: {str(e)}",
            "code": "E0002"
        }
        print(json.dumps(error_output))
        sys.exit(1)
    except Exception as e:
        error_output = {
            "error": f"Optimizer error: {str(e)}",
            "code": "E0999"
        }
        print(json.dumps(error_output))
        sys.exit(1)


if __name__ == "__main__":
    main()

