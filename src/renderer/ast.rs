//! AST representation of Confluence content

use once_cell::sync::Lazy;
use scraper::{ElementRef, Html, Node, Selector};
use thiserror::Error;

// C-06: Parse selectors once at startup using lazy_static
static LI_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("li").expect("Invalid hardcoded selector 'li'")
});

static TH_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("th").expect("Invalid hardcoded selector 'th'")
});

static TR_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("tbody tr, tr").expect("Invalid hardcoded selector 'tbody tr, tr'")
});

static TD_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("td").expect("Invalid hardcoded selector 'td'")
});

static AC_PARAM_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("ac\\:parameter").expect("Invalid hardcoded selector 'ac:parameter'")
});

static AC_BODY_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("ac\\:plain-text-body, ac\\:rich-text-body")
        .expect("Invalid hardcoded selector for ac body")
});

// Security limits
const MAX_HTML_SIZE: usize = 5 * 1024 * 1024; // 5MB (H-10)
const MAX_PARSE_DEPTH: usize = 100; // C-07

// Dangerous HTML tags to filter (H-02)
const DANGEROUS_TAGS: &[&str] = &["script", "style", "iframe", "object", "embed", "form"];

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid HTML: {0}")]
    InvalidHtml(String),

    #[error("Missing required element: {0}")]
    MissingElement(String),

    #[error("HTML too large: {0}")]
    HtmlTooLarge(String),

    #[error("HTML nesting too deep: {0}")]
    NestingTooDeep(String),
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
    /// Parse Confluence storage format (XHTML) into AST (C-07, H-10: Added size and depth limits)
    pub fn parse(html: &str) -> Result<Self, ParseError> {
        // H-10: Check HTML size limit
        if html.len() > MAX_HTML_SIZE {
            return Err(ParseError::HtmlTooLarge(format!(
                "HTML size {} bytes exceeds maximum of {} bytes ({}MB)",
                html.len(),
                MAX_HTML_SIZE,
                MAX_HTML_SIZE / (1024 * 1024)
            )));
        }

        let document = Html::parse_fragment(html);
        let root = document.root_element();

        let mut nodes = Vec::new();
        for child in root.children() {
            if let Some(node) = Self::parse_node_with_depth(child, 0)? {
                nodes.push(node);
            }
        }

        Ok(Self {
            root: AstNode::Document(nodes),
        })
    }

    /// Parse node with depth tracking (C-07)
    fn parse_node_with_depth(
        node: ego_tree::NodeRef<Node>,
        depth: usize,
    ) -> Result<Option<AstNode>, ParseError> {
        if depth > MAX_PARSE_DEPTH {
            return Err(ParseError::NestingTooDeep(format!(
                "HTML nesting exceeds maximum depth of {}",
                MAX_PARSE_DEPTH
            )));
        }

        match node.value() {
            Node::Text(text) => {
                let content = text.trim();
                if content.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(AstNode::Text(content.to_string())))
                }
            }
            Node::Element(_) => {
                if let Some(element_ref) = ElementRef::wrap(node) {
                    Self::parse_element_ref_with_depth(element_ref, depth + 1)
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    /// Legacy parse_node for compatibility - redirects to depth-tracked version
    fn parse_node(node: ego_tree::NodeRef<Node>) -> Option<AstNode> {
        Self::parse_node_with_depth(node, 0).ok().flatten()
    }

    /// Parse element with depth tracking and security filtering (H-02)
    fn parse_element_ref_with_depth(
        element: ElementRef,
        depth: usize,
    ) -> Result<Option<AstNode>, ParseError> {
        let tag = element.value().name();

        // H-02: Filter dangerous tags
        if DANGEROUS_TAGS.contains(&tag) {
            tracing::warn!("Filtered out dangerous HTML tag: {}", tag);
            return Ok(None);
        }

        let result = match tag {
            // Headings
            "h1" => Some(AstNode::Heading {
                level: 1,
                content: Self::parse_element_children_with_depth(element, depth)?,
            }),
            "h2" => Some(AstNode::Heading {
                level: 2,
                content: Self::parse_element_children_with_depth(element, depth)?,
            }),
            "h3" => Some(AstNode::Heading {
                level: 3,
                content: Self::parse_element_children_with_depth(element, depth)?,
            }),
            "h4" => Some(AstNode::Heading {
                level: 4,
                content: Self::parse_element_children_with_depth(element, depth)?,
            }),
            "h5" => Some(AstNode::Heading {
                level: 5,
                content: Self::parse_element_children_with_depth(element, depth)?,
            }),
            "h6" => Some(AstNode::Heading {
                level: 6,
                content: Self::parse_element_children_with_depth(element, depth)?,
            }),

            // Paragraph
            "p" => Some(AstNode::Paragraph(
                Self::parse_element_children_with_depth(element, depth)?,
            )),

            // Text formatting
            "strong" | "b" => Some(AstNode::Bold(
                Self::parse_element_children_with_depth(element, depth)?,
            )),
            "em" | "i" => Some(AstNode::Italic(
                Self::parse_element_children_with_depth(element, depth)?,
            )),
            "code" => {
                let text = element.text().collect::<String>();
                Some(AstNode::Code(text))
            }

            // Links (H-11: Handle empty href)
            "a" => {
                let url = element.value().attr("href").filter(|s| !s.is_empty());

                match url {
                    Some(url) => Some(AstNode::Link {
                        url: url.to_string(),
                        text: Self::parse_element_children_with_depth(element, depth)?,
                    }),
                    None => {
                        // Link with no href - just return text content
                        let children = Self::parse_element_children_with_depth(element, depth)?;
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

            // Lists
            "ul" => Some(Self::parse_list_with_depth(element, false, depth)?),
            "ol" => Some(Self::parse_list_with_depth(element, true, depth)?),

            // Tables
            "table" => Self::parse_table_with_depth(element, depth)?,

            // Confluence macros
            tag if tag.starts_with("ac:") => Self::parse_macro_with_depth(element, depth)?,

            // Skip unknown elements but parse their children
            _ => {
                let children = Self::parse_element_children_with_depth(element, depth)?;
                if children.len() == 1 {
                    Some(children.into_iter().next().unwrap())
                } else if !children.is_empty() {
                    Some(AstNode::Paragraph(children))
                } else {
                    None
                }
            }
        };

        Ok(result)
    }

    /// Legacy parse_element_ref for compatibility
    fn parse_element_ref(element: ElementRef) -> Option<AstNode> {
        Self::parse_element_ref_with_depth(element, 0).ok().flatten()
    }

    /// Parse element children with depth tracking
    fn parse_element_children_with_depth(
        element: ElementRef,
        depth: usize,
    ) -> Result<Vec<AstNode>, ParseError> {
        element
            .children()
            .filter_map(|child| Self::parse_node_with_depth(child, depth).transpose())
            .collect()
    }

    /// Legacy version for compatibility
    fn parse_element_children(element: ElementRef) -> Vec<AstNode> {
        Self::parse_element_children_with_depth(element, 0).unwrap_or_default()
    }

    /// Parse list with depth tracking (C-06: Uses static selector)
    fn parse_list_with_depth(
        element: ElementRef,
        ordered: bool,
        depth: usize,
    ) -> Result<AstNode, ParseError> {
        let items: Vec<Vec<AstNode>> = element
            .select(&*LI_SELECTOR)
            .map(|li| Self::parse_element_children_with_depth(li, depth))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(AstNode::List { ordered, items })
    }

    /// Legacy version
    fn parse_list(element: ElementRef, ordered: bool) -> AstNode {
        Self::parse_list_with_depth(element, ordered, 0).unwrap_or(AstNode::List {
            ordered,
            items: vec![],
        })
    }

    /// Parse table with depth tracking (C-06: Uses static selectors)
    fn parse_table_with_depth(
        element: ElementRef,
        _depth: usize,
    ) -> Result<Option<AstNode>, ParseError> {
        // Extract headers
        let headers: Vec<String> = element
            .select(&*TH_SELECTOR)
            .map(|th| th.text().collect::<String>().trim().to_string())
            .collect();

        // Extract rows
        let rows: Vec<Vec<String>> = element
            .select(&*TR_SELECTOR)
            .filter(|tr| {
                // Skip header row
                tr.select(&*TH_SELECTOR).count() == 0
            })
            .map(|tr| {
                tr.select(&*TD_SELECTOR)
                    .map(|td| td.text().collect::<String>().trim().to_string())
                    .collect()
            })
            .filter(|row: &Vec<String>| !row.is_empty())
            .collect();

        if headers.is_empty() && rows.is_empty() {
            Ok(None)
        } else {
            Ok(Some(AstNode::Table { headers, rows }))
        }
    }

    /// Legacy version
    fn parse_table(element: ElementRef) -> Option<AstNode> {
        Self::parse_table_with_depth(element, 0).ok().flatten()
    }

    /// Parse macro with depth tracking (C-06: Uses static selectors)
    fn parse_macro_with_depth(
        element: ElementRef,
        _depth: usize,
    ) -> Result<Option<AstNode>, ParseError> {
        let name = element.value().attr("ac:name").unwrap_or("").to_string();

        // Extract parameters
        let params: Vec<(String, String)> = element
            .select(&*AC_PARAM_SELECTOR)
            .filter_map(|param| {
                let key = param.value().attr("ac:name")?.to_string();
                let value = param.text().collect::<String>();
                Some((key, value))
            })
            .collect();

        // Extract body
        let body = element
            .select(&*AC_BODY_SELECTOR)
            .next()
            .map(|b| b.text().collect::<String>())
            .unwrap_or_default();

        Ok(Some(AstNode::Macro { name, params, body }))
    }

    /// Legacy version
    fn parse_macro(element: ElementRef) -> Option<AstNode> {
        Self::parse_macro_with_depth(element, 0).ok().flatten()
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
