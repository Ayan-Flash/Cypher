use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a node in the Abstract Syntax Tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNode {
    /// Type of the AST node
    pub node_type: AstNodeType,
    /// Text content of the node
    pub text: String,
    /// Start position (line, column)
    pub start: (usize, usize),
    /// End position (line, column)
    pub end: (usize, usize),
    /// Child nodes
    pub children: Vec<AstNode>,
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Types of AST nodes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AstNodeType {
    // Common node types
    SourceFile,
    FunctionDefinition,
    FunctionCall,
    VariableDeclaration,
    VariableAssignment,
    Identifier,
    Literal,
    BinaryExpression,
    UnaryExpression,
    Block,
    IfStatement,
    ElseClause,
    WhileLoop,
    ForLoop,
    ReturnStatement,
    Comment,
    StringLiteral,
    NumberLiteral,
    BooleanLiteral,
    
    // Language-specific node types
    StructDefinition,
    EnumDefinition,
    ImplBlock,
    TraitDefinition,
    ClassDefinition,
    InterfaceDefinition,
    ImportStatement,
    ModuleDeclaration,
    MacroDefinition,
    MacroCall,
    
    // Security-relevant node types
    SqlQuery,
    ShellCommand,
    FileOperation,
    NetworkRequest,
    CryptographicOperation,
    AuthenticationCall,
    AuthorizationCall,
    DataSerialization,
    DataDeserialization,
    
    // Generic/unknown
    Unknown,
}

impl AstNode {
    /// Create a new AST node
    pub fn new(
        node_type: AstNodeType,
        text: String,
        start: (usize, usize),
        end: (usize, usize),
    ) -> Self {
        Self {
            node_type,
            text,
            start,
            end,
            children: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a child node
    pub fn add_child(&mut self, child: AstNode) {
        self.children.push(child);
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get the line number
    pub fn line(&self) -> usize {
        self.start.0
    }

    /// Get the column number
    pub fn column(&self) -> usize {
        self.start.1
    }

    /// Get the text span
    pub fn span(&self) -> ((usize, usize), (usize, usize)) {
        (self.start, self.end)
    }

    /// Find all descendants of a specific type
    pub fn find_by_type(&self, node_type: &AstNodeType) -> Vec<&AstNode> {
        let mut results = Vec::new();
        if &self.node_type == node_type {
            results.push(self);
        }
        for child in &self.children {
            results.extend(child.find_by_type(node_type));
        }
        results
    }

    /// Find first descendant of a specific type
    pub fn find_first_by_type(&self, node_type: &AstNodeType) -> Option<&AstNode> {
        if &self.node_type == node_type {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_first_by_type(node_type) {
                return Some(found);
            }
        }
        None
    }

    /// Check if node contains a specific text pattern
    pub fn contains_text(&self, pattern: &str) -> bool {
        self.text.contains(pattern) || self.children.iter().any(|c| c.contains_text(pattern))
    }

    /// Get all text content recursively
    pub fn full_text(&self) -> String {
        let mut text = self.text.clone();
        for child in &self.children {
            text.push_str(&child.full_text());
        }
        text
    }
}

impl AstNodeType {
    /// Check if this node type is security-relevant
    pub fn is_security_relevant(&self) -> bool {
        matches!(
            self,
            AstNodeType::SqlQuery
                | AstNodeType::ShellCommand
                | AstNodeType::FileOperation
                | AstNodeType::NetworkRequest
                | AstNodeType::CryptographicOperation
                | AstNodeType::AuthenticationCall
                | AstNodeType::AuthorizationCall
                | AstNodeType::DataSerialization
                | AstNodeType::DataDeserialization
        )
    }

    /// Get description of the node type
    pub fn description(&self) -> &str {
        match self {
            AstNodeType::SourceFile => "Source file",
            AstNodeType::FunctionDefinition => "Function definition",
            AstNodeType::FunctionCall => "Function call",
            AstNodeType::VariableDeclaration => "Variable declaration",
            AstNodeType::VariableAssignment => "Variable assignment",
            AstNodeType::Identifier => "Identifier",
            AstNodeType::Literal => "Literal value",
            AstNodeType::BinaryExpression => "Binary expression",
            AstNodeType::UnaryExpression => "Unary expression",
            AstNodeType::Block => "Code block",
            AstNodeType::IfStatement => "If statement",
            AstNodeType::ElseClause => "Else clause",
            AstNodeType::WhileLoop => "While loop",
            AstNodeType::ForLoop => "For loop",
            AstNodeType::ReturnStatement => "Return statement",
            AstNodeType::Comment => "Comment",
            AstNodeType::StringLiteral => "String literal",
            AstNodeType::NumberLiteral => "Number literal",
            AstNodeType::BooleanLiteral => "Boolean literal",
            AstNodeType::StructDefinition => "Struct definition",
            AstNodeType::EnumDefinition => "Enum definition",
            AstNodeType::ImplBlock => "Implementation block",
            AstNodeType::TraitDefinition => "Trait definition",
            AstNodeType::ClassDefinition => "Class definition",
            AstNodeType::InterfaceDefinition => "Interface definition",
            AstNodeType::ImportStatement => "Import statement",
            AstNodeType::ModuleDeclaration => "Module declaration",
            AstNodeType::MacroDefinition => "Macro definition",
            AstNodeType::MacroCall => "Macro call",
            AstNodeType::SqlQuery => "SQL query",
            AstNodeType::ShellCommand => "Shell command",
            AstNodeType::FileOperation => "File operation",
            AstNodeType::NetworkRequest => "Network request",
            AstNodeType::CryptographicOperation => "Cryptographic operation",
            AstNodeType::AuthenticationCall => "Authentication call",
            AstNodeType::AuthorizationCall => "Authorization call",
            AstNodeType::DataSerialization => "Data serialization",
            AstNodeType::DataDeserialization => "Data deserialization",
            AstNodeType::Unknown => "Unknown node type",
        }
    }
}
