#![allow(dead_code)]

use crate::error::{Result, CypherError};
use crate::parser::{ast::AstNode, ast::AstNodeType, language::Language};
use std::path::Path;

/// AST Parser using tree-sitter
pub struct Parser {
    language: Language,
}

impl Parser {
    /// Create a new parser for a specific language
    pub fn new(language: Language) -> Self {
        Self { language }
    }

    /// Detect language from file path and create parser
    pub fn from_file_path(file_path: &Path) -> Result<Self> {
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| CypherError::Ast("No file extension found".to_string()))?;

        let language = Language::from_extension(extension)
            .ok_or_else(|| CypherError::Ast(format!("Unsupported language: {}", extension)))?;

        Ok(Self::new(language))
    }

    /// Parse source code into an AST
    pub fn parse(&self, source: &str) -> Result<AstNode> {
        let mut parser = tree_sitter::Parser::new();

        // Load the appropriate language grammar
        let tree_sitter_lang = self.load_tree_sitter_language()?;
        parser.set_language(&tree_sitter_lang)
            .map_err(|e| CypherError::Ast(format!("Failed to set language: {}", e)))?;

        // Parse the source code
        let tree = parser
            .parse(source, None)
            .ok_or_else(|| CypherError::Ast("Failed to parse source code".to_string()))?;

        // Convert tree-sitter tree to our AST representation
        self.convert_tree_to_ast(&tree, source)
    }

    /// Load the appropriate tree-sitter language
    fn load_tree_sitter_language(&self) -> Result<tree_sitter::Language> {
        match self.language {
            Language::Rust => Ok(tree_sitter_rust::language()),
            Language::JavaScript | Language::TypeScript => Ok(tree_sitter_javascript::language()),
            Language::Python => Ok(tree_sitter_python::language()),
            Language::Go => Ok(tree_sitter_go::language()),
            _ => Err(CypherError::Ast(format!(
                "Tree-sitter grammar not yet implemented for {}",
                self.language
            ))),
        }
    }

    /// Convert tree-sitter tree to our AST representation
    fn convert_tree_to_ast(&self, tree: &tree_sitter::Tree, source: &str) -> Result<AstNode> {
        let root_node = tree.root_node();
        self.convert_node(&root_node, source)
    }

    /// Convert a single tree-sitter node to our AST node
    fn convert_node(&self, node: &tree_sitter::Node, source: &str) -> Result<AstNode> {
        let node_type = self.map_node_type(node.kind());
        let text = node.utf8_text(source.as_bytes())
            .unwrap_or("")
            .to_string();

        let start_byte = node.start_byte();
        let end_byte = node.end_byte();
        
        // Calculate line and column positions
        let start_line = source[..start_byte].lines().count();
        let start_col = source[..start_byte].lines().last().map_or(0, |l| l.len());
        let end_line = source[..end_byte].lines().count();
        let end_col = source[..end_byte].lines().last().map_or(0, |l| l.len());

        let mut ast_node = AstNode::new(
            node_type,
            text,
            (start_line, start_col),
            (end_line, end_col),
        );

        // Add metadata
        ast_node.add_metadata("ts_kind".to_string(), node.kind().to_string());

        // Recursively convert children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            ast_node.add_child(self.convert_node(&child, source)?);
        }

        Ok(ast_node)
    }

    /// Map tree-sitter node kind to our AST node type
    fn map_node_type(&self, kind: &str) -> AstNodeType {
        match kind {
            // Common patterns
            "source_file" => AstNodeType::SourceFile,
            "function_definition" | "function_declaration" | "function_item" => {
                AstNodeType::FunctionDefinition
            }
            "call_expression" | "function_call" => AstNodeType::FunctionCall,
            "variable_declaration" | "let_declaration" | "const_declaration" => {
                AstNodeType::VariableDeclaration
            }
            "assignment_expression" | "assignment" => AstNodeType::VariableAssignment,
            "identifier" => AstNodeType::Identifier,
            "string" | "string_literal" => AstNodeType::StringLiteral,
            "number" | "integer" | "float" => AstNodeType::NumberLiteral,
            "true" | "false" => AstNodeType::BooleanLiteral,
            "block" | "block_statement" => AstNodeType::Block,
            "if_expression" | "if_statement" => AstNodeType::IfStatement,
            "else" | "else_clause" => AstNodeType::ElseClause,
            "while_expression" | "while_statement" => AstNodeType::WhileLoop,
            "for_expression" | "for_statement" | "for_in_expression" => AstNodeType::ForLoop,
            "return_statement" => AstNodeType::ReturnStatement,
            "comment" | "line_comment" | "block_comment" => AstNodeType::Comment,
            
            // Rust-specific
            "struct_item" | "struct_definition" => AstNodeType::StructDefinition,
            "enum_item" | "enum_definition" => AstNodeType::EnumDefinition,
            "impl_item" | "impl_block" => AstNodeType::ImplBlock,
            "trait_item" | "trait_definition" => AstNodeType::TraitDefinition,
            "macro_definition" => AstNodeType::MacroDefinition,
            "macro_invocation" | "macro_call" => AstNodeType::MacroCall,
            "mod_item" | "module_declaration" => AstNodeType::ModuleDeclaration,
            "use_declaration" => AstNodeType::ImportStatement,
            
            // JavaScript/TypeScript-specific
            "class_declaration" => AstNodeType::ClassDefinition,
            "interface_declaration" => AstNodeType::InterfaceDefinition,
            "import_statement" | "export_statement" => AstNodeType::ImportStatement,
            
            // Python-specific
            "import_from_statement" => AstNodeType::ImportStatement,
            
            // Go-specific
            "type_declaration" => AstNodeType::StructDefinition,
            "import_declaration" => AstNodeType::ImportStatement,
            
            // Security-relevant patterns (heuristic-based)
            _ if kind.contains("sql") || kind.contains("query") => AstNodeType::SqlQuery,
            _ if kind.contains("shell") || kind.contains("command") => AstNodeType::ShellCommand,
            _ if kind.contains("file") || kind.contains("fs") => AstNodeType::FileOperation,
            _ if kind.contains("http") || kind.contains("request") => AstNodeType::NetworkRequest,
            _ if kind.contains("crypto") || kind.contains("hash") => AstNodeType::CryptographicOperation,
            _ if kind.contains("auth") => AstNodeType::AuthenticationCall,
            
            // Default
            _ => AstNodeType::Unknown,
        }
    }

    /// Get the language of this parser
    pub fn language(&self) -> &Language {
        &self.language
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = Parser::new(Language::Rust);
        assert_eq!(parser.language(), &Language::Rust);
    }

    #[test]
    fn test_rust_code_parsing() {
        let parser = Parser::new(Language::Rust);
        let source = r#"
fn main() {
    println!("Hello, world!");
}
"#;

        let result = parser.parse(source);
        assert!(result.is_ok());
        
        let ast = result.unwrap();
        assert_eq!(ast.node_type, AstNodeType::SourceFile);
    }

    #[test]
    fn test_javascript_code_parsing() {
        let parser = Parser::new(Language::JavaScript);
        let source = r#"
function hello() {
    console.log("Hello, world!");
}
"#;

        let result = parser.parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_file_path() {
        let parser = Parser::from_file_path(Path::new("test.rs"));
        assert!(parser.is_ok());
        assert_eq!(parser.unwrap().language(), &Language::Rust);
    }
}
