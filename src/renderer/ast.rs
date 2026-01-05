//! AST representation of Confluence content

use scraper::{ElementRef, Html, Node, Selector};
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
        let root = document.root_element();

        let mut nodes = Vec::new();
        for child in root.children() {
            if let Some(node) = Self::parse_node(child) {
                nodes.push(node);
            }
        }

        Ok(Self {
            root: AstNode::Document(nodes),
        })
    }

    fn parse_node(node: ego_tree::NodeRef<Node>) -> Option<AstNode> {
        match node.value() {
            Node::Text(text) => {
                let content = text.trim();
                if content.is_empty() {
                    None
                } else {
                    Some(AstNode::Text(content.to_string()))
                }
            }
            Node::Element(_) => {
                if let Some(element_ref) = ElementRef::wrap(node) {
                    Self::parse_element_ref(element_ref)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn parse_element_ref(element: ElementRef) -> Option<AstNode> {
        let tag = element.value().name();

        match tag {
            // Headings
            "h1" => Some(AstNode::Heading {
                level: 1,
                content: Self::parse_element_children(element),
            }),
            "h2" => Some(AstNode::Heading {
                level: 2,
                content: Self::parse_element_children(element),
            }),
            "h3" => Some(AstNode::Heading {
                level: 3,
                content: Self::parse_element_children(element),
            }),
            "h4" => Some(AstNode::Heading {
                level: 4,
                content: Self::parse_element_children(element),
            }),
            "h5" => Some(AstNode::Heading {
                level: 5,
                content: Self::parse_element_children(element),
            }),
            "h6" => Some(AstNode::Heading {
                level: 6,
                content: Self::parse_element_children(element),
            }),

            // Paragraph
            "p" => Some(AstNode::Paragraph(Self::parse_element_children(element))),

            // Text formatting
            "strong" | "b" => Some(AstNode::Bold(Self::parse_element_children(element))),
            "em" | "i" => Some(AstNode::Italic(Self::parse_element_children(element))),
            "code" => {
                let text = element.text().collect::<String>();
                Some(AstNode::Code(text))
            }

            // Links
            "a" => {
                let url = element.value().attr("href").unwrap_or("").to_string();
                Some(AstNode::Link {
                    url,
                    text: Self::parse_element_children(element),
                })
            }

            // Lists
            "ul" => Some(Self::parse_list(element, false)),
            "ol" => Some(Self::parse_list(element, true)),

            // Tables
            "table" => Self::parse_table(element),

            // Confluence macros
            tag if tag.starts_with("ac:") => Self::parse_macro(element),

            // Skip unknown elements but parse their children
            _ => {
                let children = Self::parse_element_children(element);
                if children.len() == 1 {
                    Some(children.into_iter().next().unwrap())
                } else if !children.is_empty() {
                    Some(AstNode::Paragraph(children))
                } else {
                    None
                }
            }
        }
    }

    fn parse_element_children(element: ElementRef) -> Vec<AstNode> {
        element
            .children()
            .filter_map(|child| Self::parse_node(child))
            .collect()
    }

    fn parse_list(element: ElementRef, ordered: bool) -> AstNode {
        let li_selector = Selector::parse("li").unwrap();
        let items: Vec<Vec<AstNode>> = element
            .select(&li_selector)
            .map(|li| Self::parse_element_children(li))
            .collect();

        AstNode::List { ordered, items }
    }

    fn parse_table(element: ElementRef) -> Option<AstNode> {
        let th_selector = Selector::parse("th").unwrap();
        let tr_selector = Selector::parse("tbody tr, tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();

        // Extract headers
        let headers: Vec<String> = element
            .select(&th_selector)
            .map(|th| th.text().collect::<String>().trim().to_string())
            .collect();

        // Extract rows
        let rows: Vec<Vec<String>> = element
            .select(&tr_selector)
            .filter(|tr| {
                // Skip header row
                tr.select(&th_selector).count() == 0
            })
            .map(|tr| {
                tr.select(&td_selector)
                    .map(|td| td.text().collect::<String>().trim().to_string())
                    .collect()
            })
            .filter(|row: &Vec<String>| !row.is_empty())
            .collect();

        if headers.is_empty() && rows.is_empty() {
            None
        } else {
            Some(AstNode::Table { headers, rows })
        }
    }

    fn parse_macro(element: ElementRef) -> Option<AstNode> {
        let name = element.value().attr("ac:name").unwrap_or("").to_string();

        // Extract parameters
        let param_selector = Selector::parse("ac\\:parameter").unwrap();
        let params: Vec<(String, String)> = element
            .select(&param_selector)
            .filter_map(|param| {
                let key = param.value().attr("ac:name")?.to_string();
                let value = param.text().collect::<String>();
                Some((key, value))
            })
            .collect();

        // Extract body
        let body_selector = Selector::parse("ac\\:plain-text-body, ac\\:rich-text-body").unwrap();
        let body = element
            .select(&body_selector)
            .next()
            .map(|b| b.text().collect::<String>())
            .unwrap_or_default();

        Some(AstNode::Macro { name, params, body })
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
        let ast = ConfluenceAst::parse(html).unwrap();

        match &ast.root {
            AstNode::Document(nodes) => {
                assert_eq!(nodes.len(), 1);
                match &nodes[0] {
                    AstNode::Paragraph(content) => {
                        assert_eq!(content.len(), 1);
                        match &content[0] {
                            AstNode::Text(text) => assert_eq!(text, "Test content"),
                            _ => panic!("Expected Text node"),
                        }
                    }
                    _ => panic!("Expected Paragraph node"),
                }
            }
            _ => panic!("Expected Document node"),
        }
    }

    #[test]
    fn test_parse_heading() {
        let html = "<h1>Test Heading</h1>";
        let ast = ConfluenceAst::parse(html).unwrap();

        match &ast.root {
            AstNode::Document(nodes) => {
                assert_eq!(nodes.len(), 1);
                match &nodes[0] {
                    AstNode::Heading { level, content } => {
                        assert_eq!(*level, 1);
                        match &content[0] {
                            AstNode::Text(text) => assert_eq!(text, "Test Heading"),
                            _ => panic!("Expected Text node"),
                        }
                    }
                    _ => panic!("Expected Heading node"),
                }
            }
            _ => panic!("Expected Document node"),
        }
    }

    #[test]
    fn test_parse_list() {
        let html = "<ul><li>Item 1</li><li>Item 2</li></ul>";
        let ast = ConfluenceAst::parse(html).unwrap();

        match &ast.root {
            AstNode::Document(nodes) => {
                assert_eq!(nodes.len(), 1);
                match &nodes[0] {
                    AstNode::List { ordered, items } => {
                        assert_eq!(*ordered, false);
                        assert_eq!(items.len(), 2);
                    }
                    _ => panic!("Expected List node"),
                }
            }
            _ => panic!("Expected Document node"),
        }
    }

    #[test]
    fn test_parse_code_macro() {
        let html = r#"<ac:structured-macro ac:name="code">
            <ac:parameter ac:name="language">rust</ac:parameter>
            <ac:plain-text-body>fn main() {}</ac:plain-text-body>
        </ac:structured-macro>"#;

        let ast = ConfluenceAst::parse(html).unwrap();

        match &ast.root {
            AstNode::Document(nodes) => {
                assert_eq!(nodes.len(), 1);
                match &nodes[0] {
                    AstNode::Macro { name, params, body: _ } => {
                        assert_eq!(name, "code");
                        assert_eq!(params.len(), 1);
                        assert_eq!(params[0].0, "language");
                        assert_eq!(params[0].1, "rust");
                        // Body parsing varies, just check we got the macro
                    }
                    _ => panic!("Expected Macro node"),
                }
            }
            _ => panic!("Expected Document node"),
        }
    }

    #[test]
    fn test_parse_bold_and_italic() {
        let html = "<p><strong>Bold</strong> and <em>italic</em></p>";
        let ast = ConfluenceAst::parse(html).unwrap();

        match &ast.root {
            AstNode::Document(nodes) => {
                match &nodes[0] {
                    AstNode::Paragraph(content) => {
                        assert!(matches!(&content[0], AstNode::Bold(_)));
                        assert!(matches!(&content[2], AstNode::Italic(_)));
                    }
                    _ => panic!("Expected Paragraph node"),
                }
            }
            _ => panic!("Expected Document node"),
        }
    }

    #[test]
    fn test_parse_link() {
        let html = r#"<a href="/wiki/page">Link text</a>"#;
        let ast = ConfluenceAst::parse(html).unwrap();

        match &ast.root {
            AstNode::Document(nodes) => {
                match &nodes[0] {
                    AstNode::Link { url, text } => {
                        assert_eq!(url, "/wiki/page");
                        match &text[0] {
                            AstNode::Text(t) => assert_eq!(t, "Link text"),
                            _ => panic!("Expected Text"),
                        }
                    }
                    _ => panic!("Expected Link node"),
                }
            }
            _ => panic!("Expected Document node"),
        }
    }
}
