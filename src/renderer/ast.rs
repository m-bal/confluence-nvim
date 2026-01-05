//! AST representation of Confluence content

use scraper::{Html, Selector};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid HTML: {0}")]
    InvalidHtml(String),

    #[error("Missing required element: {0}")]
    MissingElement(String),
}

/// Abstract Syntax Tree for Confluence content
#[derive(Debug, Clone)]
pub struct ConfluenceAst {
    pub root: AstNode,
}

/// AST node types
#[derive(Debug, Clone)]
pub enum AstNode {
    Document(Vec<AstNode>),
    Heading { level: u8, content: Vec<AstNode> },
    Paragraph(Vec<AstNode>),
    Text(String),
    Bold(Vec<AstNode>),
    Italic(Vec<AstNode>),
    Code(String),
    Link { url: String, text: Vec<AstNode> },
    List { ordered: bool, items: Vec<Vec<AstNode>> },
    Table { headers: Vec<String>, rows: Vec<Vec<String>> },
    Macro { name: String, params: Vec<(String, String)>, body: String },
}

impl ConfluenceAst {
    /// Parse Confluence storage format (XHTML) into AST
    pub fn parse(html: &str) -> Result<Self, ParseError> {
        let document = Html::parse_fragment(html);

        // TODO: Implement actual parsing logic
        // For now, return empty document
        Ok(Self {
            root: AstNode::Document(vec![]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let result = ConfluenceAst::parse("");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_simple_paragraph() {
        let html = "<p>Test content</p>";
        let result = ConfluenceAst::parse(html);
        assert!(result.is_ok());
    }
}
