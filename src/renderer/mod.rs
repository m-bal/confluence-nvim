//! Content rendering module
//!
//! Converts Confluence storage format (XHTML) to rich terminal output
//! using Neovim's extmarks, virtual text, and highlighting features.

use thiserror::Error;

mod ast;
mod elements;
mod highlights;

pub use ast::{AstNode, ConfluenceAst};
pub use elements::{CodeBlock, MacroPanel, MacroType, Table};
pub use highlights::{ConfluenceHighlights, HighlightGroup};

/// Default box width for rendering
const DEFAULT_BOX_WIDTH: usize = 70;

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Failed to parse content: {0}")]
    ParseError(String),

    #[error("Unsupported element: {0}")]
    UnsupportedElement(String),

    #[error("Invalid content structure: {0}")]
    InvalidStructure(String),
}

/// Rendered content ready for display in Neovim
#[derive(Debug, Clone)]
pub struct RenderedContent {
    /// Text content lines
    pub lines: Vec<String>,

    /// Highlight groups to apply
    pub highlights: Vec<HighlightGroup>,

    /// Metadata about the rendered content
    pub metadata: RenderMetadata,

    /// Extracted links with their positions
    pub links: Vec<LinkInfo>,
}

/// Information about a link in the rendered content
#[derive(Debug, Clone)]
pub struct LinkInfo {
    /// Line number (0-indexed)
    pub line: usize,
    /// Column start (0-indexed)
    pub col_start: usize,
    /// Column end (0-indexed)
    pub col_end: usize,
    /// Target URL
    pub url: String,
    /// Display text
    pub text: String,
}

/// Metadata about rendered content
#[derive(Debug, Clone)]
pub struct RenderMetadata {
    /// Page ID
    pub page_id: String,

    /// Page title
    pub title: String,

    /// Total line count
    pub line_count: usize,
}

/// Main renderer for Confluence content
pub struct Renderer {
    /// Enable syntax highlighting for code blocks
    pub enable_syntax_highlighting: bool,

    /// Enable image rendering
    pub enable_images: bool,

    /// Maximum image width in characters
    pub max_image_width: u32,

    /// Box width for panels and code blocks
    pub box_width: usize,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            enable_syntax_highlighting: true,
            enable_images: true,
            max_image_width: 80,
            box_width: DEFAULT_BOX_WIDTH,
        }
    }
}

impl Renderer {
    /// Create a new renderer with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Render Confluence storage format to terminal output
    pub fn render(
        &self,
        page_id: &str,
        title: &str,
        content: &str,
    ) -> Result<RenderedContent, RenderError> {
        // Parse XHTML to AST
        let ast = ConfluenceAst::parse(content)
            .map_err(|e| RenderError::ParseError(e.to_string()))?;

        // Render AST to lines and highlights
        let mut ctx = RenderContext::new(self.box_width);

        // Render the document
        self.render_node(&ast.root, &mut ctx);

        // Clean up excessive blank lines
        let lines = self.clean_lines(ctx.lines);

        Ok(RenderedContent {
            metadata: RenderMetadata {
                page_id: page_id.to_string(),
                title: title.to_string(),
                line_count: lines.len(),
            },
            lines,
            highlights: ctx.highlights,
            links: ctx.links,
        })
    }

    /// Render a single AST node
    fn render_node(&self, node: &AstNode, ctx: &mut RenderContext) {
        match node {
            AstNode::Document(children) => {
                for child in children {
                    self.render_node(child, ctx);
                }
            }
            AstNode::Heading { level, content } => {
                ctx.push_blank_line();
                let prefix = "#".repeat(*level as usize);
                let text = self.collect_text(content);
                ctx.push_line(format!("{} {}", prefix, text));
                ctx.push_blank_line();
            }
            AstNode::Paragraph(children) => {
                let text = self.collect_inline_nodes(children, ctx);
                if !text.trim().is_empty() {
                    ctx.push_line(text);
                    ctx.push_blank_line();
                }
            }
            AstNode::Text(text) => {
                // Text nodes are typically collected by parent
                ctx.push_line(text.clone());
            }
            AstNode::Bold(children) => {
                let text = self.collect_text(children);
                ctx.push_line(format!("**{}**", text));
            }
            AstNode::Italic(children) => {
                let text = self.collect_text(children);
                ctx.push_line(format!("*{}*", text));
            }
            AstNode::Code(text) => {
                ctx.push_line(format!("`{}`", text));
            }
            AstNode::Link { url, text } => {
                let link_text = self.collect_text(text);
                // Mark Confluence internal links with arrow
                if url.contains("/wiki/spaces/") || url.contains("/pages/") {
                    ctx.push_line(format!("{} →", link_text));
                } else {
                    ctx.push_line(link_text);
                }
            }
            AstNode::List { ordered, items } => {
                ctx.push_blank_line();
                for (idx, item) in items.iter().enumerate() {
                    let prefix = if *ordered {
                        format!("{}.", idx + 1)
                    } else {
                        "•".to_string()
                    };
                    let text = self.collect_text(item);
                    ctx.push_line(format!("  {} {}", prefix, text));
                }
                ctx.push_blank_line();
            }
            AstNode::Table { headers, rows } => {
                let table = Table {
                    headers: headers.clone(),
                    rows: rows.clone(),
                };
                ctx.push_blank_line();
                for line in table.render() {
                    ctx.push_line(line);
                }
                ctx.push_blank_line();
            }
            AstNode::Macro { name, params, body } => {
                self.render_macro(name, params, body, ctx);
            }
        }
    }

    /// Render a Confluence macro
    fn render_macro(
        &self,
        name: &str,
        params: &[(String, String)],
        body: &str,
        ctx: &mut RenderContext,
    ) {
        match name {
            "info" => {
                let panel = MacroPanel {
                    macro_type: MacroType::Info,
                    content: body.trim().to_string(),
                };
                ctx.push_blank_line();
                self.render_panel(&panel, ctx);
                ctx.push_blank_line();
            }
            "warning" => {
                let panel = MacroPanel {
                    macro_type: MacroType::Warning,
                    content: body.trim().to_string(),
                };
                ctx.push_blank_line();
                self.render_panel(&panel, ctx);
                ctx.push_blank_line();
            }
            "note" => {
                let panel = MacroPanel {
                    macro_type: MacroType::Note,
                    content: body.trim().to_string(),
                };
                ctx.push_blank_line();
                self.render_panel(&panel, ctx);
                ctx.push_blank_line();
            }
            "tip" | "success" => {
                let panel = MacroPanel {
                    macro_type: MacroType::Success,
                    content: body.trim().to_string(),
                };
                ctx.push_blank_line();
                self.render_panel(&panel, ctx);
                ctx.push_blank_line();
            }
            "error" => {
                let panel = MacroPanel {
                    macro_type: MacroType::Error,
                    content: body.trim().to_string(),
                };
                ctx.push_blank_line();
                self.render_panel(&panel, ctx);
                ctx.push_blank_line();
            }
            "code" | "noformat" => {
                let language = params
                    .iter()
                    .find(|(k, _)| k == "language")
                    .map(|(_, v)| v.clone())
                    .unwrap_or_else(|| "text".to_string());

                let code_block = CodeBlock {
                    language,
                    code: body.trim().to_string(),
                    line_numbers: self.enable_syntax_highlighting,
                };
                ctx.push_blank_line();
                self.render_code_block(&code_block, ctx);
                ctx.push_blank_line();
            }
            "expand" => {
                // Expand macro - render as collapsible section indicator
                let title = params
                    .iter()
                    .find(|(k, _)| k == "title")
                    .map(|(_, v)| v.clone())
                    .unwrap_or_else(|| "Details".to_string());
                ctx.push_blank_line();
                ctx.push_line(format!("▶ {} ───────────────────────────────", title));
                if !body.is_empty() {
                    for line in body.lines() {
                        ctx.push_line(format!("  {}", line));
                    }
                }
                ctx.push_line("──────────────────────────────────────────".to_string());
                ctx.push_blank_line();
            }
            "toc" => {
                // Table of contents - render placeholder
                ctx.push_blank_line();
                ctx.push_line("📑 [Table of Contents]".to_string());
                ctx.push_blank_line();
            }
            "anchor" => {
                // Anchor macro - invisible, skip
            }
            "status" => {
                // Status macro - inline badge
                let color = params
                    .iter()
                    .find(|(k, _)| k == "colour" || k == "color")
                    .map(|(_, v)| v.clone())
                    .unwrap_or_else(|| "Grey".to_string());
                let title = params
                    .iter()
                    .find(|(k, _)| k == "title")
                    .map(|(_, v)| v.clone())
                    .unwrap_or_else(|| "Status".to_string());
                ctx.push_line(format!("[{}:{}]", color.to_uppercase(), title));
            }
            _ => {
                // Unknown macro - render body as-is with indicator
                if !body.is_empty() {
                    ctx.push_blank_line();
                    ctx.push_line(format!("┌─ {} ────────────────────────────────", name.to_uppercase()));
                    for line in body.lines() {
                        ctx.push_line(format!("│ {}", line));
                    }
                    ctx.push_line("└────────────────────────────────────────────".to_string());
                    ctx.push_blank_line();
                }
            }
        }
    }

    /// Render a macro panel with proper box drawing
    fn render_panel(&self, panel: &MacroPanel, ctx: &mut RenderContext) {
        let width = ctx.box_width;
        let icon = panel.icon();
        let label = panel.label();

        // Calculate header width
        let header_text = format!("{} {}", icon, label);
        let remaining = width.saturating_sub(header_text.len() + 4);

        ctx.push_line(format!(
            "┌─ {} {}┐",
            header_text,
            "─".repeat(remaining)
        ));

        // Wrap content lines to fit box
        for line in panel.content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                ctx.push_line(format!("│{}│", " ".repeat(width)));
            } else {
                // Word wrap long lines
                for wrapped in self.wrap_text(trimmed, width - 2) {
                    let padding = width - wrapped.len() - 2;
                    ctx.push_line(format!("│ {}{} │", wrapped, " ".repeat(padding.max(0))));
                }
            }
        }

        ctx.push_line(format!("└{}┘", "─".repeat(width)));
    }

    /// Render a code block with proper box drawing
    fn render_code_block(&self, block: &CodeBlock, ctx: &mut RenderContext) {
        let width = ctx.box_width;
        let lang_upper = block.language.to_uppercase();

        // Header
        let header_text = format!("─ {} ", lang_upper);
        let remaining = width.saturating_sub(header_text.len() + 1);
        ctx.push_line(format!("┌{}{}┐", header_text, "─".repeat(remaining)));

        // Code lines with optional line numbers
        let code_lines: Vec<&str> = block.code.lines().collect();
        let line_num_width = if block.line_numbers {
            code_lines.len().to_string().len().max(3)
        } else {
            0
        };

        for (idx, line) in code_lines.iter().enumerate() {
            let content = if block.line_numbers {
                format!("{:>width$} │ {}", idx + 1, line, width = line_num_width)
            } else {
                format!(" {}", line)
            };

            // Truncate if too long
            let display_content = if content.len() > width - 2 {
                format!("{}…", &content[..width - 3])
            } else {
                content
            };

            let padding = width - display_content.len() - 2;
            ctx.push_line(format!("│{}{} │", display_content, " ".repeat(padding.max(0))));
        }

        // Footer
        ctx.push_line(format!("└{}┘", "─".repeat(width)));
    }

    /// Collect text from inline nodes (for paragraphs)
    fn collect_inline_nodes(&self, nodes: &[AstNode], ctx: &mut RenderContext) -> String {
        let mut parts = Vec::new();
        for node in nodes {
            match node {
                AstNode::Text(text) => parts.push(text.clone()),
                AstNode::Bold(children) => {
                    let text = self.collect_text(children);
                    parts.push(format!("**{}**", text));
                }
                AstNode::Italic(children) => {
                    let text = self.collect_text(children);
                    parts.push(format!("*{}*", text));
                }
                AstNode::Code(text) => {
                    parts.push(format!("`{}`", text));
                }
                AstNode::Link { url, text } => {
                    let link_text = self.collect_text(text);
                    let current_line = ctx.lines.len();
                    let col_start = parts.iter().map(|p| p.len()).sum::<usize>();

                    // Track link position
                    ctx.links.push(LinkInfo {
                        line: current_line,
                        col_start,
                        col_end: col_start + link_text.len(),
                        url: url.clone(),
                        text: link_text.clone(),
                    });

                    // Mark Confluence internal links
                    if url.contains("/wiki/spaces/") || url.contains("/pages/") {
                        parts.push(format!("{} →", link_text));
                    } else {
                        parts.push(link_text);
                    }
                }
                _ => {
                    // For other nodes, just collect their text
                    parts.push(self.collect_text(&[node.clone()]));
                }
            }
        }
        parts.join("")
    }

    /// Recursively collect plain text from nodes
    fn collect_text(&self, nodes: &[AstNode]) -> String {
        let mut result = String::new();
        for node in nodes {
            match node {
                AstNode::Text(text) => result.push_str(text),
                AstNode::Bold(children) | AstNode::Italic(children) => {
                    result.push_str(&self.collect_text(children));
                }
                AstNode::Code(text) => result.push_str(text),
                AstNode::Link { url, text } => {
                    result.push_str(&self.collect_text(text));
                    // Add arrow for Confluence internal links
                    if url.contains("/wiki/spaces/") || url.contains("/pages/") {
                        result.push_str(" →");
                    }
                }
                AstNode::Paragraph(children) => {
                    result.push_str(&self.collect_text(children));
                }
                _ => {}
            }
        }
        result
    }

    /// Wrap text to fit within a given width
    fn wrap_text(&self, text: &str, max_width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in text.split_whitespace() {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_line.len() + 1 + word.len() <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        if lines.is_empty() {
            lines.push(String::new());
        }

        lines
    }

    /// Clean up excessive blank lines
    fn clean_lines(&self, lines: Vec<String>) -> Vec<String> {
        let mut result = Vec::new();
        let mut prev_blank = false;

        for line in lines {
            let is_blank = line.trim().is_empty();
            if is_blank {
                if !prev_blank {
                    result.push(String::new());
                }
                prev_blank = true;
            } else {
                result.push(line);
                prev_blank = false;
            }
        }

        // Trim leading/trailing blank lines
        while result.first().map(|s| s.is_empty()).unwrap_or(false) {
            result.remove(0);
        }
        while result.last().map(|s| s.is_empty()).unwrap_or(false) {
            result.pop();
        }

        result
    }
}

/// Rendering context to accumulate output
struct RenderContext {
    lines: Vec<String>,
    highlights: Vec<HighlightGroup>,
    links: Vec<LinkInfo>,
    box_width: usize,
}

impl RenderContext {
    fn new(box_width: usize) -> Self {
        Self {
            lines: Vec::new(),
            highlights: Vec::new(),
            links: Vec::new(),
            box_width,
        }
    }

    fn push_line(&mut self, line: String) {
        self.lines.push(line);
    }

    fn push_blank_line(&mut self) {
        self.lines.push(String::new());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let renderer = Renderer::new();
        assert!(renderer.enable_syntax_highlighting);
        assert!(renderer.enable_images);
        assert_eq!(renderer.max_image_width, 80);
        assert_eq!(renderer.box_width, DEFAULT_BOX_WIDTH);
    }

    #[test]
    fn test_render_paragraph() {
        let renderer = Renderer::new();
        let result = renderer.render("123", "Test Page", "<p>Hello World</p>");

        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert_eq!(rendered.metadata.page_id, "123");
        assert_eq!(rendered.metadata.title, "Test Page");
        assert!(rendered.lines.iter().any(|l| l.contains("Hello World")));
    }

    #[test]
    fn test_render_heading() {
        let renderer = Renderer::new();
        let result = renderer.render("123", "Test", "<h1>Main Title</h1><h2>Sub Title</h2>");

        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.lines.iter().any(|l| l.contains("# Main Title")));
        assert!(rendered.lines.iter().any(|l| l.contains("## Sub Title")));
    }

    #[test]
    fn test_render_list() {
        let renderer = Renderer::new();
        let result = renderer.render("123", "Test", "<ul><li>Item 1</li><li>Item 2</li></ul>");

        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.lines.iter().any(|l| l.contains("• Item 1")));
        assert!(rendered.lines.iter().any(|l| l.contains("• Item 2")));
    }

    #[test]
    fn test_render_ordered_list() {
        let renderer = Renderer::new();
        let result = renderer.render("123", "Test", "<ol><li>First</li><li>Second</li></ol>");

        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.lines.iter().any(|l| l.contains("1. First")));
        assert!(rendered.lines.iter().any(|l| l.contains("2. Second")));
    }

    #[test]
    fn test_render_table() {
        let renderer = Renderer::new();
        let result = renderer.render(
            "123",
            "Test",
            "<table><tr><th>Name</th><th>Value</th></tr><tr><td>Foo</td><td>Bar</td></tr></table>",
        );

        assert!(result.is_ok());
        let rendered = result.unwrap();
        // Check for box-drawing characters
        assert!(rendered.lines.iter().any(|l| l.contains("┌")));
        assert!(rendered.lines.iter().any(|l| l.contains("│")));
        assert!(rendered.lines.iter().any(|l| l.contains("└")));
        // Check for content
        assert!(rendered.lines.iter().any(|l| l.contains("Name")));
        assert!(rendered.lines.iter().any(|l| l.contains("Foo")));
    }

    #[test]
    fn test_render_code_macro() {
        let renderer = Renderer::new();
        let result = renderer.render(
            "123",
            "Test",
            r#"<ac:structured-macro ac:name="code">
                <ac:parameter ac:name="language">rust</ac:parameter>
                <ac:plain-text-body>fn main() {}</ac:plain-text-body>
            </ac:structured-macro>"#,
        );

        assert!(result.is_ok());
        let rendered = result.unwrap();
        // Check for RUST header
        assert!(rendered.lines.iter().any(|l| l.contains("RUST")));
        // Check for code content
        assert!(rendered.lines.iter().any(|l| l.contains("fn main()")));
        // Check for box-drawing
        assert!(rendered.lines.iter().any(|l| l.contains("┌")));
    }

    #[test]
    fn test_render_info_macro() {
        let renderer = Renderer::new();
        let result = renderer.render(
            "123",
            "Test",
            r#"<ac:structured-macro ac:name="info">
                <ac:rich-text-body>Important information here</ac:rich-text-body>
            </ac:structured-macro>"#,
        );

        assert!(result.is_ok());
        let rendered = result.unwrap();
        // Check for INFO panel with icon
        assert!(rendered.lines.iter().any(|l| l.contains("ℹ") && l.contains("INFO")));
        // Check for content
        assert!(rendered.lines.iter().any(|l| l.contains("Important information")));
    }

    #[test]
    fn test_render_warning_macro() {
        let renderer = Renderer::new();
        let result = renderer.render(
            "123",
            "Test",
            r#"<ac:structured-macro ac:name="warning">
                <ac:rich-text-body>Warning message</ac:rich-text-body>
            </ac:structured-macro>"#,
        );

        assert!(result.is_ok());
        let rendered = result.unwrap();
        // Check for WARNING panel with icon
        assert!(rendered.lines.iter().any(|l| l.contains("⚠") && l.contains("WARNING")));
    }

    #[test]
    fn test_render_inline_code() {
        let renderer = Renderer::new();
        let result = renderer.render("123", "Test", "<p>Use <code>println!</code> macro</p>");

        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.lines.iter().any(|l| l.contains("`println!`")));
    }

    #[test]
    fn test_render_bold_italic() {
        let renderer = Renderer::new();
        let result = renderer.render(
            "123",
            "Test",
            "<p><strong>Bold</strong> and <em>italic</em></p>",
        );

        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.lines.iter().any(|l| l.contains("**Bold**")));
        assert!(rendered.lines.iter().any(|l| l.contains("*italic*")));
    }

    #[test]
    fn test_render_confluence_link() {
        let renderer = Renderer::new();
        let result = renderer.render(
            "123",
            "Test",
            r#"<p><a href="/wiki/spaces/TEAM/pages/12345">Related Page</a></p>"#,
        );

        assert!(result.is_ok());
        let rendered = result.unwrap();
        // Check for arrow indicator for Confluence links
        assert!(rendered.lines.iter().any(|l| l.contains("Related Page →")));
        // Check link was tracked
        assert!(!rendered.links.is_empty());
    }

    #[test]
    fn test_clean_excessive_blank_lines() {
        let renderer = Renderer::new();
        let result = renderer.render(
            "123",
            "Test",
            "<p>First</p><p></p><p></p><p></p><p>Second</p>",
        );

        assert!(result.is_ok());
        let rendered = result.unwrap();

        // Count consecutive blank lines - should never be more than 1
        let mut prev_blank = false;
        let mut double_blank_count = 0;
        for line in &rendered.lines {
            if line.is_empty() {
                if prev_blank {
                    double_blank_count += 1;
                }
                prev_blank = true;
            } else {
                prev_blank = false;
            }
        }
        assert_eq!(double_blank_count, 0, "Should not have consecutive blank lines");
    }

    #[test]
    fn test_render_empty_content() {
        let renderer = Renderer::new();
        let result = renderer.render("123", "Test", "");

        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.lines.is_empty() || rendered.lines.iter().all(|l| l.is_empty()));
    }

    #[test]
    fn test_render_complex_document() {
        let renderer = Renderer::new();
        let html = r#"
            <h1>Project Documentation</h1>
            <p>Welcome to the project docs.</p>
            <ac:structured-macro ac:name="info">
                <ac:rich-text-body>Read carefully!</ac:rich-text-body>
            </ac:structured-macro>
            <h2>Getting Started</h2>
            <ul>
                <li>Install dependencies</li>
                <li>Run setup</li>
            </ul>
            <ac:structured-macro ac:name="code">
                <ac:parameter ac:name="language">bash</ac:parameter>
                <ac:plain-text-body>npm install</ac:plain-text-body>
            </ac:structured-macro>
        "#;

        let result = renderer.render("123", "Project Docs", html);

        assert!(result.is_ok());
        let rendered = result.unwrap();

        // Verify all major elements are present
        assert!(rendered.lines.iter().any(|l| l.contains("# Project Documentation")));
        assert!(rendered.lines.iter().any(|l| l.contains("Welcome")));
        assert!(rendered.lines.iter().any(|l| l.contains("ℹ") && l.contains("INFO")));
        assert!(rendered.lines.iter().any(|l| l.contains("## Getting Started")));
        assert!(rendered.lines.iter().any(|l| l.contains("• Install")));
        assert!(rendered.lines.iter().any(|l| l.contains("BASH")));
        assert!(rendered.lines.iter().any(|l| l.contains("npm install")));
    }

    // =========================================================================
    // EDGE CASE TESTS
    // =========================================================================

    #[test]
    fn test_whitespace_only_content() {
        let renderer = Renderer::new();
        let result = renderer.render("123", "Test", "   \n\t\n   ");
        assert!(result.is_ok());
    }

    #[test]
    fn test_unicode_content() {
        let renderer = Renderer::new();
        let html = "<p>日本語テスト 中文测试 한국어테스트</p>";
        let result = renderer.render("123", "Unicode", html);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.lines.iter().any(|l| l.contains("日本語")));
    }

    #[test]
    fn test_emoji_content() {
        let renderer = Renderer::new();
        let html = "<p>🎉 Celebration! 🚀 Launch</p>";
        let result = renderer.render("123", "Emoji", html);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.lines.iter().any(|l| l.contains("🎉")));
    }

    #[test]
    fn test_very_long_line() {
        let renderer = Renderer::new();
        let long_text = "A".repeat(5000);
        let html = format!("<p>{}</p>", long_text);
        let result = renderer.render("123", "Long", &html);
        assert!(result.is_ok());
    }

    #[test]
    fn test_many_paragraphs() {
        let renderer = Renderer::new();
        let paragraphs: String = (0..100)
            .map(|i| format!("<p>Para {}</p>", i))
            .collect();
        let result = renderer.render("123", "Many", &paragraphs);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.lines.iter().any(|l| l.contains("Para 0")));
        assert!(rendered.lines.iter().any(|l| l.contains("Para 99")));
    }

    #[test]
    fn test_empty_tags() {
        let renderer = Renderer::new();
        let html = "<p></p><div></div><span></span>";
        let result = renderer.render("123", "Empty", html);
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_inline_formatting() {
        let renderer = Renderer::new();
        let html = "<p><strong><em><code>deep</code></em></strong></p>";
        let result = renderer.render("123", "Nested", html);
        assert!(result.is_ok());
    }

    // =========================================================================
    // COMPLEX STRUCTURE TESTS
    // =========================================================================

    #[test]
    fn test_nested_lists() {
        let renderer = Renderer::new();
        let html = r#"<ul><li>A<ul><li>Nested</li></ul></li><li>B</li></ul>"#;
        let result = renderer.render("123", "Nested Lists", html);
        assert!(result.is_ok());
    }

    #[test]
    fn test_table_empty_cells() {
        let renderer = Renderer::new();
        let html = "<table><tr><th>H</th></tr><tr><td></td></tr><tr><td>Data</td></tr></table>";
        let result = renderer.render("123", "Empty Cells", html);
        assert!(result.is_ok());
    }

    #[test]
    fn test_heading_hierarchy() {
        let renderer = Renderer::new();
        let html = "<h1>H1</h1><h2>H2</h2><h3>H3</h3><h4>H4</h4><h5>H5</h5><h6>H6</h6>";
        let result = renderer.render("123", "Headings", html);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.lines.iter().any(|l| l.starts_with("# H1")));
        assert!(rendered.lines.iter().any(|l| l.starts_with("###### H6")));
    }

    // =========================================================================
    // LINK TRACKING TESTS
    // =========================================================================

    #[test]
    fn test_multiple_links() {
        let renderer = Renderer::new();
        let html = r#"<p>
            <a href="/wiki/spaces/A/pages/1">L1</a>
            <a href="/wiki/spaces/B/pages/2">L2</a>
            <a href="/wiki/spaces/C/pages/3">L3</a>
        </p>"#;
        let result = renderer.render("123", "Links", html);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.links.len() >= 3);
    }

    #[test]
    fn test_external_link_no_arrow() {
        let renderer = Renderer::new();
        let html = r#"<p><a href="https://example.com">External</a></p>"#;
        let result = renderer.render("123", "External", html);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        let combined = rendered.lines.join(" ");
        assert!(combined.contains("External"));
        assert!(!combined.contains("External →"));
    }

    #[test]
    fn test_internal_link_has_arrow() {
        let renderer = Renderer::new();
        let html = r#"<p><a href="/wiki/spaces/T/pages/1">Internal</a></p>"#;
        let result = renderer.render("123", "Internal", html);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.lines.iter().any(|l| l.contains("Internal →")));
    }

    #[test]
    fn test_empty_href() {
        let renderer = Renderer::new();
        let html = r#"<p><a href="">Empty</a></p>"#;
        let result = renderer.render("123", "Empty Href", html);
        assert!(result.is_ok());
    }

    // =========================================================================
    // MACRO TESTS
    // =========================================================================

    #[test]
    fn test_note_macro() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="note">
            <ac:rich-text-body>Note content</ac:rich-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "Note", html);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.lines.iter().any(|l| l.contains("📝")));
        assert!(rendered.lines.iter().any(|l| l.contains("NOTE")));
    }

    #[test]
    fn test_tip_macro() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="tip">
            <ac:rich-text-body>Tip</ac:rich-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "Tip", html);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.lines.iter().any(|l| l.contains("SUCCESS")));
    }

    #[test]
    fn test_error_macro() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="error">
            <ac:rich-text-body>Error</ac:rich-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "Error", html);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.lines.iter().any(|l| l.contains("ERROR")));
    }

    #[test]
    fn test_code_without_language() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="code">
            <ac:plain-text-body>code</ac:plain-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "NoLang", html);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.lines.iter().any(|l| l.contains("TEXT")));
    }

    #[test]
    fn test_expand_macro() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="expand">
            <ac:parameter ac:name="title">Details</ac:parameter>
            <ac:rich-text-body>Hidden</ac:rich-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "Expand", html);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.lines.iter().any(|l| l.contains("▶")));
    }

    #[test]
    fn test_toc_macro() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="toc"></ac:structured-macro>"#;
        let result = renderer.render("123", "TOC", html);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.lines.iter().any(|l| l.contains("📑")));
    }

    #[test]
    fn test_status_macro() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="status">
            <ac:parameter ac:name="colour">Green</ac:parameter>
            <ac:parameter ac:name="title">Done</ac:parameter>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "Status", html);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.lines.iter().any(|l| l.contains("[GREEN:Done]")));
    }

    #[test]
    fn test_unknown_macro() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="custom">
            <ac:rich-text-body>Content</ac:rich-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "Custom", html);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.lines.iter().any(|l| l.contains("CUSTOM")));
    }

    #[test]
    fn test_anchor_invisible() {
        let renderer = Renderer::new();
        let html = r#"<p>Before</p>
            <ac:structured-macro ac:name="anchor">
                <ac:parameter ac:name="name">anchor1</ac:parameter>
            </ac:structured-macro>
            <p>After</p>"#;
        let result = renderer.render("123", "Anchor", html);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.lines.iter().any(|l| l.contains("Before")));
        assert!(rendered.lines.iter().any(|l| l.contains("After")));
    }

    // =========================================================================
    // SECURITY TESTS
    // =========================================================================

    #[test]
    fn test_script_filtered() {
        let renderer = Renderer::new();
        let html = "<p>Safe</p><script>alert('xss')</script><p>Also safe</p>";
        let result = renderer.render("123", "Script", html);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        let combined = rendered.lines.join(" ");
        assert!(!combined.contains("alert"));
        assert!(combined.contains("Safe"));
    }

    #[test]
    fn test_style_filtered() {
        let renderer = Renderer::new();
        let html = "<style>body{background:red}</style><p>Content</p>";
        let result = renderer.render("123", "Style", html);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        let combined = rendered.lines.join(" ");
        assert!(!combined.contains("background"));
    }

    #[test]
    fn test_iframe_filtered() {
        let renderer = Renderer::new();
        let html = r#"<p>Safe</p><iframe src="https://evil.com"></iframe>"#;
        let result = renderer.render("123", "Iframe", html);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        let combined = rendered.lines.join(" ");
        assert!(!combined.contains("evil.com"));
    }

    // =========================================================================
    // BOX DRAWING TESTS
    // =========================================================================

    #[test]
    fn test_panel_box_structure() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="info">
            <ac:rich-text-body>Test</ac:rich-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "Box", html);
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert!(rendered.lines.iter().any(|l| l.contains("┌")));
        assert!(rendered.lines.iter().any(|l| l.contains("┐")));
        assert!(rendered.lines.iter().any(|l| l.contains("└")));
        assert!(rendered.lines.iter().any(|l| l.contains("┘")));
        assert!(rendered.lines.iter().any(|l| l.contains("│")));
    }

    #[test]
    fn test_custom_box_width() {
        let mut renderer = Renderer::new();
        renderer.box_width = 40;
        let html = r#"<ac:structured-macro ac:name="info">
            <ac:rich-text-body>Short</ac:rich-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "Width", html);
        assert!(result.is_ok());
    }

    // =========================================================================
    // METADATA TESTS
    // =========================================================================

    #[test]
    fn test_metadata_unicode_title() {
        let renderer = Renderer::new();
        let result = renderer.render("123", "日本語タイトル", "<p>Content</p>");
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert_eq!(rendered.metadata.title, "日本語タイトル");
    }

    #[test]
    fn test_metadata_special_id() {
        let renderer = Renderer::new();
        let result = renderer.render("abc-123_xyz", "Test", "<p>Content</p>");
        assert!(result.is_ok());
        let rendered = result.unwrap();
        assert_eq!(rendered.metadata.page_id, "abc-123_xyz");
    }

    // =========================================================================
    // INTEGRATION TESTS
    // =========================================================================

    #[test]
    fn test_realistic_api_doc_page() {
        let renderer = Renderer::new();
        let html = r#"
            <h1>API Documentation</h1>
            <p>REST API endpoints.</p>
            <ac:structured-macro ac:name="toc"></ac:structured-macro>
            <h2>Authentication</h2>
            <ac:structured-macro ac:name="info">
                <ac:rich-text-body>Bearer token required.</ac:rich-text-body>
            </ac:structured-macro>
            <h3>Get Token</h3>
            <p>POST to <code>/api/auth</code></p>
            <ac:structured-macro ac:name="code">
                <ac:parameter ac:name="language">bash</ac:parameter>
                <ac:plain-text-body>curl -X POST /api/auth</ac:plain-text-body>
            </ac:structured-macro>
            <h2>Endpoints</h2>
            <table>
                <tr><th>Method</th><th>Path</th></tr>
                <tr><td>GET</td><td>/users</td></tr>
                <tr><td>POST</td><td>/users</td></tr>
            </table>
            <ac:structured-macro ac:name="warning">
                <ac:rich-text-body>Rate limited.</ac:rich-text-body>
            </ac:structured-macro>
            <h2>See Also</h2>
            <ul>
                <li><a href="/wiki/spaces/A/pages/1">Errors</a></li>
                <li><a href="https://github.com">GitHub</a></li>
            </ul>
        "#;

        let result = renderer.render("789", "API Docs", html);
        assert!(result.is_ok());
        let rendered = result.unwrap();

        // Verify structure
        assert!(rendered.lines.iter().any(|l| l.contains("# API Documentation")));
        assert!(rendered.lines.iter().any(|l| l.contains("📑")));
        assert!(rendered.lines.iter().any(|l| l.contains("## Authentication")));
        assert!(rendered.lines.iter().any(|l| l.contains("ℹ")));
        assert!(rendered.lines.iter().any(|l| l.contains("### Get Token")));
        assert!(rendered.lines.iter().any(|l| l.contains("`/api/auth`")));
        assert!(rendered.lines.iter().any(|l| l.contains("BASH")));
        assert!(rendered.lines.iter().any(|l| l.contains("## Endpoints")));
        assert!(rendered.lines.iter().any(|l| l.contains("GET")));
        assert!(rendered.lines.iter().any(|l| l.contains("⚠")));
        // Internal link should have arrow indicator
        assert!(rendered.lines.iter().any(|l| l.contains("Errors") && l.contains("→")));

        // External link (GitHub) should NOT have arrow
        let combined = rendered.lines.join("\n");
        assert!(combined.contains("GitHub"));
        // GitHub link should appear without arrow (external link)
        assert!(!combined.contains("GitHub →"));
    }

    #[test]
    fn test_renderer_reuse() {
        let renderer = Renderer::new();

        let r1 = renderer.render("1", "First", "<p>First</p>").unwrap();
        let r2 = renderer.render("2", "Second", "<p>Second</p>").unwrap();

        assert_eq!(r1.metadata.page_id, "1");
        assert_eq!(r2.metadata.page_id, "2");
    }

    #[test]
    fn test_malformed_html_graceful() {
        let renderer = Renderer::new();
        let html = "<p>Unclosed<div>Mixed <b>tags</p></div>";
        let result = renderer.render("123", "Malformed", html);
        assert!(result.is_ok());
    }
}
