/**
 * FAX Compiler - Shared Type Definitions
 * 
 * This module defines common TypeScript interfaces and types used throughout
 * the FAX compiler pipeline for type-safe communication between components.
 */

/**
 * Represents a single lexical token produced by the lexer.
 * Includes the token type, value, and position information for error reporting.
 */
export interface Token {
  /** Token type (Keyword, Identifier, String, Number, etc.) */
  type: string;
  /** The actual string value of the token */
  value: string;
  /** Line number where token appears (1-indexed) */
  line: number;
  /** Column number where token appears (1-indexed) */
  column: number;
}

/**
 * Represents an Abstract Syntax Tree (AST) node.
 * All AST nodes have a 'type' field identifying the node kind.
 * Additional properties depend on the specific node type.
 */
export interface ASTNode {
  /** The type of AST node (Program, FunctionDeclaration, BinaryExpression, etc.) */
  type: string;
  /** Additional properties specific to this node type */
  [key: string]: any;
}

/**
 * Configuration passed to the compiler hub from the entry point.
 * Controls compilation behavior and target selection.
 */
export interface CompilerConfig {
  /** Path to the source file to compile */
  sourcePath: string;
  /** Optional output file path (default: source filename with .out extension) */
  outputPath?: string;
  /** Target language for code generation */
  targetLanguage: 'rs' | 'zig' | 'cpp' | 'py' | 'hs';
}

