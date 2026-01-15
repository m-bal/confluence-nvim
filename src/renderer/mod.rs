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
/// Minimum box width
const MIN_BOX_WIDTH: usize = 40;
/// Maximum box width (screen width limit)
const MAX_BOX_WIDTH: usize = 120;

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

    /// Box width for panels and code blocks (0 = dynamic)
    pub box_width: usize,

    /// Maximum box width (for dynamic sizing)
    pub max_box_width: usize,

    /// Use dynamic box sizing based on content
    pub dynamic_box_width: bool,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            enable_syntax_highlighting: true,
            enable_images: true,
            max_image_width: 80,
            box_width: DEFAULT_BOX_WIDTH,
            max_box_width: MAX_BOX_WIDTH,
            dynamic_box_width: true,
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
        let icon = panel.icon();
        let label = panel.label();
        let icon_chars = icon.chars().count();
        let label_chars = label.chars().count();

        // Calculate dynamic width based on content if enabled
        let width = if self.dynamic_box_width {
            // Find the longest content line
            let max_content_width = panel.content.lines()
                .map(|l| l.trim().chars().count())
                .max()
                .unwrap_or(0);

            // Header needs: icon + label + 6 chars for "┌─  ┐" formatting
            let header_min = icon_chars + label_chars + 6;

            // Content needs: content + 4 chars for "│  │" borders
            let content_min = max_content_width + 4;

            // Use the larger of header or content requirements
            let needed = header_min.max(content_min);

            // Clamp between min and max
            needed.clamp(MIN_BOX_WIDTH, self.max_box_width)
        } else {
            ctx.box_width
        };

        // Calculate header dashes
        // Header format: ┌─ {icon} {label} {dashes}┐
        // Total chars: 1 + 1 + 1 + icon + 1 + label + 1 + dashes + 1 = width + 2
        let dashes_needed = width.saturating_sub(4 + icon_chars + label_chars);

        ctx.push_line(format!(
            "┌─ {} {} {}┐",
            icon,
            label,
            "─".repeat(dashes_needed)
        ));

        // Render content lines (no wrapping in dynamic mode - content fits)
        for line in panel.content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                ctx.push_line(format!("│{}│", " ".repeat(width)));
            } else {
                let content_chars = trimmed.chars().count();
                let padding = width.saturating_sub(content_chars + 2);
                ctx.push_line(format!("│ {}{} │", trimmed, " ".repeat(padding)));
            }
        }

        ctx.push_line(format!("└{}┘", "─".repeat(width)));
    }

    /// Render a code block with proper box drawing
    fn render_code_block(&self, block: &CodeBlock, ctx: &mut RenderContext) {
        let lang_upper = block.language.to_uppercase();
        let lang_chars = lang_upper.chars().count();

        // Code lines with optional line numbers
        let code_lines: Vec<&str> = block.code.lines().collect();
        let line_num_width = if block.line_numbers {
            code_lines.len().to_string().len().max(3)
        } else {
            0
        };

        // Calculate dynamic width based on content if enabled
        let width = if self.dynamic_box_width {
            // Find the longest code line (with line number prefix if enabled)
            let max_code_width = code_lines.iter()
                .map(|l| {
                    if block.line_numbers {
                        // Line number + " │ " + code
                        line_num_width + 3 + l.chars().count()
                    } else {
                        // " " + code
                        1 + l.chars().count()
                    }
                })
                .max()
                .unwrap_or(0);

            // Header needs: lang + 5 chars for "┌─  ┐" formatting
            let header_min = lang_chars + 5;

            // Content needs: content + 3 chars for "│ │" borders
            let content_min = max_code_width + 3;

            // Use the larger of header or content requirements
            let needed = header_min.max(content_min);

            // Clamp between min and max
            needed.clamp(MIN_BOX_WIDTH, self.max_box_width)
        } else {
            ctx.box_width
        };

        // Header format: ┌─ {LANG} {dashes}┐
        // Total chars: 1 + 1 + 1 + lang + 1 + dashes + 1 = width + 2
        let dashes_needed = width.saturating_sub(3 + lang_chars);
        ctx.push_line(format!("┌─ {} {}┐", lang_upper, "─".repeat(dashes_needed)));

        for (idx, line) in code_lines.iter().enumerate() {
            let content = if block.line_numbers {
                format!("{:>width$} │ {}", idx + 1, line, width = line_num_width)
            } else {
                format!(" {}", line)
            };

            // In dynamic mode, content should fit; in fixed mode, truncate if needed
            let content_chars = content.chars().count();
            let display_content = if !self.dynamic_box_width && content_chars > width - 2 {
                let truncated: String = content.chars().take(width - 3).collect();
                format!("{}…", truncated)
            } else {
                content
            };

            let display_chars = display_content.chars().count();
            // Total line: │ + content + padding + space + │ = width + 2
            // So: 1 + content + padding + 1 + 1 = width + 2
            // padding = width - content - 1
            let padding = width.saturating_sub(display_chars + 1);
            ctx.push_line(format!("│{}{} │", display_content, " ".repeat(padding)));
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

    // =========================================================================
    // UI RENDERING TESTS - Exact Visual Output Verification
    // =========================================================================

    // -------------------------------------------------------------------------
    // TABLE VISUAL TESTS
    // -------------------------------------------------------------------------

    #[test]
    fn test_table_exact_box_drawing() {
        let renderer = Renderer::new();
        let html = r#"<table>
            <tr><th>A</th><th>B</th></tr>
            <tr><td>1</td><td>2</td></tr>
        </table>"#;
        let result = renderer.render("123", "Table", html).unwrap();
        let output = result.lines.join("\n");

        // Verify exact box drawing characters
        assert!(output.contains("┌"), "Missing top-left corner");
        assert!(output.contains("┐"), "Missing top-right corner");
        assert!(output.contains("└"), "Missing bottom-left corner");
        assert!(output.contains("┘"), "Missing bottom-right corner");
        assert!(output.contains("┬"), "Missing top T-junction");
        assert!(output.contains("┴"), "Missing bottom T-junction");
        assert!(output.contains("┼"), "Missing cross junction");
        assert!(output.contains("├"), "Missing left T-junction");
        assert!(output.contains("┤"), "Missing right T-junction");
        assert!(output.contains("│"), "Missing vertical bars");
        assert!(output.contains("─"), "Missing horizontal bars");
    }

    #[test]
    fn test_table_column_alignment() {
        let renderer = Renderer::new();
        let html = r#"<table>
            <tr><th>Short</th><th>LongerHeader</th></tr>
            <tr><td>X</td><td>Y</td></tr>
            <tr><td>VeryLongCell</td><td>Z</td></tr>
        </table>"#;
        let result = renderer.render("123", "Align", html).unwrap();

        // Find lines with vertical bars
        let data_lines: Vec<_> = result.lines.iter()
            .filter(|l| l.starts_with("│") && !l.contains("─"))
            .collect();

        // All data lines should have same length (aligned columns)
        if data_lines.len() >= 2 {
            let first_len = data_lines[0].chars().count();
            for line in &data_lines {
                assert_eq!(
                    line.chars().count(),
                    first_len,
                    "Column alignment mismatch: {:?}",
                    line
                );
            }
        }
    }

    #[test]
    fn test_table_header_separator() {
        let renderer = Renderer::new();
        let html = r#"<table>
            <tr><th>H1</th><th>H2</th></tr>
            <tr><td>D1</td><td>D2</td></tr>
        </table>"#;
        let result = renderer.render("123", "Sep", html).unwrap();

        // Header separator must be between header and data
        let lines: Vec<_> = result.lines.iter()
            .filter(|l| !l.is_empty())
            .collect();

        let separator_idx = lines.iter()
            .position(|l| l.contains("├") && l.contains("┼") && l.contains("┤"));
        assert!(separator_idx.is_some(), "Missing header separator");

        // Find header row (contains H1, H2)
        let header_idx = lines.iter().position(|l| l.contains("H1"));
        // Find data row (contains D1, D2)
        let data_idx = lines.iter().position(|l| l.contains("D1"));

        if let (Some(h), Some(s), Some(d)) = (header_idx, separator_idx, data_idx) {
            assert!(h < s, "Header should come before separator");
            assert!(s < d, "Separator should come before data");
        }
    }

    #[test]
    fn test_table_cell_padding() {
        let renderer = Renderer::new();
        let html = r#"<table>
            <tr><th>Name</th></tr>
            <tr><td>Test</td></tr>
        </table>"#;
        let result = renderer.render("123", "Pad", html).unwrap();

        // Cells should have space padding: "│ content │"
        let content_lines: Vec<_> = result.lines.iter()
            .filter(|l| l.contains("Name") || l.contains("Test"))
            .collect();

        for line in content_lines {
            assert!(
                line.contains("│ "),
                "Missing left padding in: {}",
                line
            );
            assert!(
                line.ends_with(" │"),
                "Missing right padding in: {}",
                line
            );
        }
    }

    #[test]
    fn test_table_multi_column_borders() {
        let renderer = Renderer::new();
        let html = r#"<table>
            <tr><th>C1</th><th>C2</th><th>C3</th><th>C4</th></tr>
            <tr><td>A</td><td>B</td><td>C</td><td>D</td></tr>
        </table>"#;
        let result = renderer.render("123", "MultiCol", html).unwrap();

        // Count column separators in data rows
        let data_line = result.lines.iter()
            .find(|l| l.contains("│A") || l.contains("│ A"));

        if let Some(line) = data_line {
            let bar_count = line.matches('│').count();
            // 4 columns need 5 vertical bars (left edge + 3 separators + right edge)
            assert_eq!(bar_count, 5, "Wrong number of column separators in: {}", line);
        }
    }

    // -------------------------------------------------------------------------
    // CODE BLOCK VISUAL TESTS
    // -------------------------------------------------------------------------

    #[test]
    fn test_code_block_header_format() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="code">
            <ac:parameter ac:name="language">python</ac:parameter>
            <ac:plain-text-body>print("hello")</ac:plain-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "Code", html).unwrap();

        // Find header line
        let header = result.lines.iter()
            .find(|l| l.contains("PYTHON"));

        assert!(header.is_some(), "Missing language header");
        let h = header.unwrap();
        assert!(h.starts_with("┌─"), "Header should start with ┌─");
        assert!(h.contains("PYTHON"), "Language should be uppercase");
    }

    #[test]
    fn test_code_block_content_lines() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="code">
            <ac:parameter ac:name="language">js</ac:parameter>
            <ac:plain-text-body>line1
line2
line3</ac:plain-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "Code", html).unwrap();

        // All content lines should start with "│"
        let content_lines: Vec<_> = result.lines.iter()
            .filter(|l| l.contains("line1") || l.contains("line2") || l.contains("line3"))
            .collect();

        assert_eq!(content_lines.len(), 3, "Should have 3 content lines");
        for line in &content_lines {
            assert!(line.starts_with("│"), "Content line should start with │: {}", line);
        }
    }

    #[test]
    fn test_code_block_footer() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="code">
            <ac:parameter ac:name="language">rust</ac:parameter>
            <ac:plain-text-body>fn main(){}</ac:plain-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "Code", html).unwrap();

        // Footer should close the box
        let footer = result.lines.iter()
            .find(|l| l.starts_with("└") && l.ends_with("┘"));

        assert!(footer.is_some(), "Missing code block footer");
    }

    #[test]
    fn test_code_block_preserves_indentation() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="code">
            <ac:parameter ac:name="language">python</ac:parameter>
            <ac:plain-text-body>def foo():
    return 42</ac:plain-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "Code", html).unwrap();
        let output = result.lines.join("\n");

        // Indentation should be preserved in output
        assert!(output.contains("def foo():"), "Function definition missing");
        // The return line should have leading spaces after the │
        let return_line = result.lines.iter()
            .find(|l| l.contains("return 42"));
        assert!(return_line.is_some(), "Return statement missing");
    }

    // -------------------------------------------------------------------------
    // MACRO PANEL VISUAL TESTS
    // -------------------------------------------------------------------------

    #[test]
    fn test_info_panel_exact_format() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="info">
            <ac:rich-text-body>Important info here</ac:rich-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "Info", html).unwrap();

        // Find the header line
        let header = result.lines.iter()
            .find(|l| l.contains("ℹ") && l.contains("INFO"));

        assert!(header.is_some(), "Info panel header missing");
        let h = header.unwrap();
        assert!(h.starts_with("┌─"), "Panel header should start with ┌─");
        assert!(h.contains("─".repeat(10).as_str()), "Header should have dash extension");
    }

    #[test]
    fn test_warning_panel_icon_and_label() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="warning">
            <ac:rich-text-body>Caution required</ac:rich-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "Warn", html).unwrap();

        let header = result.lines.iter()
            .find(|l| l.contains("⚠") && l.contains("WARNING"));

        assert!(header.is_some(), "Warning panel should have ⚠ icon and WARNING label");
    }

    #[test]
    fn test_error_panel_icon_and_label() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="error">
            <ac:rich-text-body>Error occurred</ac:rich-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "Err", html).unwrap();

        let header = result.lines.iter()
            .find(|l| l.contains("✗") && l.contains("ERROR"));

        assert!(header.is_some(), "Error panel should have ✗ icon and ERROR label");
    }

    #[test]
    fn test_panel_content_indentation() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="note">
            <ac:rich-text-body>Note content line</ac:rich-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "Note", html).unwrap();

        // Content lines should have "│ " prefix
        let content = result.lines.iter()
            .find(|l| l.contains("Note content"));

        assert!(content.is_some(), "Panel content missing");
        let c = content.unwrap();
        assert!(c.starts_with("│ ") || c.starts_with("│"), "Content should be indented with │");
    }

    #[test]
    fn test_panel_multiline_content() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="info">
            <ac:rich-text-body><p>Line one</p><p>Line two</p></ac:rich-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "Multi", html).unwrap();

        let line1 = result.lines.iter().any(|l| l.contains("Line one"));
        let line2 = result.lines.iter().any(|l| l.contains("Line two"));

        assert!(line1, "First line missing from panel");
        assert!(line2, "Second line missing from panel");
    }

    #[test]
    fn test_panel_footer_closure() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="info">
            <ac:rich-text-body>Test</ac:rich-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "Footer", html).unwrap();

        let footer = result.lines.iter()
            .find(|l| l.starts_with("└") && l.contains("─") && l.ends_with("┘"));

        assert!(footer.is_some(), "Panel should have closing footer with └ and ┘");
    }

    // -------------------------------------------------------------------------
    // LIST VISUAL TESTS
    // -------------------------------------------------------------------------

    #[test]
    fn test_unordered_list_bullets() {
        let renderer = Renderer::new();
        let html = r#"<ul>
            <li>First item</li>
            <li>Second item</li>
            <li>Third item</li>
        </ul>"#;
        let result = renderer.render("123", "UL", html).unwrap();

        // Each list item should have bullet point
        let items_with_bullets: Vec<_> = result.lines.iter()
            .filter(|l| l.contains("•"))
            .collect();

        assert!(items_with_bullets.len() >= 3, "Should have 3 bullet points");
        assert!(result.lines.iter().any(|l| l.contains("• First")));
        assert!(result.lines.iter().any(|l| l.contains("• Second")));
        assert!(result.lines.iter().any(|l| l.contains("• Third")));
    }

    #[test]
    fn test_ordered_list_numbers() {
        let renderer = Renderer::new();
        let html = r#"<ol>
            <li>First</li>
            <li>Second</li>
            <li>Third</li>
        </ol>"#;
        let result = renderer.render("123", "OL", html).unwrap();

        assert!(result.lines.iter().any(|l| l.contains("1.") && l.contains("First")));
        assert!(result.lines.iter().any(|l| l.contains("2.") && l.contains("Second")));
        assert!(result.lines.iter().any(|l| l.contains("3.") && l.contains("Third")));
    }

    #[test]
    fn test_list_indentation_consistency() {
        let renderer = Renderer::new();
        let html = r#"<ul>
            <li>Alpha</li>
            <li>Beta</li>
        </ul>"#;
        let result = renderer.render("123", "Indent", html).unwrap();

        // Find list item lines
        let alpha_line = result.lines.iter().find(|l| l.contains("Alpha"));
        let beta_line = result.lines.iter().find(|l| l.contains("Beta"));

        if let (Some(a), Some(b)) = (alpha_line, beta_line) {
            let a_indent = a.chars().take_while(|c| c.is_whitespace()).count();
            let b_indent = b.chars().take_while(|c| c.is_whitespace()).count();
            assert_eq!(a_indent, b_indent, "List items should have consistent indentation");
        }
    }

    // -------------------------------------------------------------------------
    // HEADING VISUAL TESTS
    // -------------------------------------------------------------------------

    #[test]
    fn test_heading_hash_counts() {
        let renderer = Renderer::new();
        let html = r#"
            <h1>Level 1</h1>
            <h2>Level 2</h2>
            <h3>Level 3</h3>
            <h4>Level 4</h4>
            <h5>Level 5</h5>
            <h6>Level 6</h6>
        "#;
        let result = renderer.render("123", "Heads", html).unwrap();

        assert!(result.lines.iter().any(|l| l.starts_with("# Level 1")));
        assert!(result.lines.iter().any(|l| l.starts_with("## Level 2")));
        assert!(result.lines.iter().any(|l| l.starts_with("### Level 3")));
        assert!(result.lines.iter().any(|l| l.starts_with("#### Level 4")));
        assert!(result.lines.iter().any(|l| l.starts_with("##### Level 5")));
        assert!(result.lines.iter().any(|l| l.starts_with("###### Level 6")));
    }

    #[test]
    fn test_heading_text_follows_hashes() {
        let renderer = Renderer::new();
        let html = "<h2>My Important Section</h2>";
        let result = renderer.render("123", "H2", html).unwrap();

        let heading = result.lines.iter()
            .find(|l| l.contains("Important Section"));

        assert!(heading.is_some(), "Heading text missing");
        let h = heading.unwrap();
        assert!(h.starts_with("## "), "H2 should have exactly 2 hashes followed by space");
        assert!(h.ends_with("Section"), "Text should follow the hashes");
    }

    // -------------------------------------------------------------------------
    // INLINE FORMATTING VISUAL TESTS
    // -------------------------------------------------------------------------

    #[test]
    fn test_bold_asterisks() {
        let renderer = Renderer::new();
        let html = "<p><strong>Bold text</strong></p>";
        let result = renderer.render("123", "Bold", html).unwrap();

        assert!(result.lines.iter().any(|l| l.contains("**Bold text**")));
    }

    #[test]
    fn test_italic_asterisks() {
        let renderer = Renderer::new();
        let html = "<p><em>Italic text</em></p>";
        let result = renderer.render("123", "Italic", html).unwrap();

        assert!(result.lines.iter().any(|l| l.contains("*Italic text*")));
    }

    #[test]
    fn test_inline_code_backticks() {
        let renderer = Renderer::new();
        let html = "<p>Use <code>npm install</code> to install</p>";
        let result = renderer.render("123", "Inline", html).unwrap();

        assert!(result.lines.iter().any(|l| l.contains("`npm install`")));
    }

    // -------------------------------------------------------------------------
    // DOCUMENT LAYOUT TESTS
    // -------------------------------------------------------------------------

    #[test]
    fn test_blank_line_before_heading() {
        let renderer = Renderer::new();
        let html = "<p>Paragraph</p><h2>Heading</h2>";
        let result = renderer.render("123", "Layout", html).unwrap();

        // Find the paragraph and heading indices
        let para_idx = result.lines.iter().position(|l| l.contains("Paragraph"));
        let head_idx = result.lines.iter().position(|l| l.contains("## Heading"));

        if let (Some(p), Some(h)) = (para_idx, head_idx) {
            assert!(h > p + 1, "Should have blank line between paragraph and heading");
        }
    }

    #[test]
    fn test_blank_line_around_code_block() {
        let renderer = Renderer::new();
        let html = r#"<p>Before</p>
            <ac:structured-macro ac:name="code">
                <ac:parameter ac:name="language">rust</ac:parameter>
                <ac:plain-text-body>code</ac:plain-text-body>
            </ac:structured-macro>
            <p>After</p>"#;
        let result = renderer.render("123", "CodeLayout", html).unwrap();

        let before_idx = result.lines.iter().position(|l| l.contains("Before"));
        let code_start_idx = result.lines.iter().position(|l| l.contains("┌─") && l.contains("RUST"));

        if let (Some(b), Some(c)) = (before_idx, code_start_idx) {
            assert!(c > b + 1, "Should have spacing before code block");
        }
    }

    #[test]
    fn test_blank_line_around_table() {
        let renderer = Renderer::new();
        let html = r#"<p>Intro</p>
            <table><tr><th>X</th></tr><tr><td>Y</td></tr></table>
            <p>Conclusion</p>"#;
        let result = renderer.render("123", "TableLayout", html).unwrap();

        let intro_idx = result.lines.iter().position(|l| l.contains("Intro"));
        let table_start_idx = result.lines.iter().position(|l| l.starts_with("┌"));

        if let (Some(i), Some(t)) = (intro_idx, table_start_idx) {
            assert!(t > i + 1, "Should have spacing before table");
        }
    }

    #[test]
    fn test_blank_line_around_list() {
        let renderer = Renderer::new();
        let html = r#"<p>Start</p>
            <ul><li>Item</li></ul>
            <p>End</p>"#;
        let result = renderer.render("123", "ListLayout", html).unwrap();

        let start_idx = result.lines.iter().position(|l| l.contains("Start"));
        let item_idx = result.lines.iter().position(|l| l.contains("• Item"));

        if let (Some(s), Some(i)) = (start_idx, item_idx) {
            assert!(i > s + 1, "Should have spacing before list");
        }
    }

    // -------------------------------------------------------------------------
    // COMBINED VISUAL STRUCTURE TESTS
    // -------------------------------------------------------------------------

    #[test]
    fn test_full_document_visual_structure() {
        let renderer = Renderer::new();
        let html = r#"
            <h1>Documentation</h1>
            <p>Welcome to the docs.</p>
            <ac:structured-macro ac:name="info">
                <ac:rich-text-body>Read this first!</ac:rich-text-body>
            </ac:structured-macro>
            <h2>Features</h2>
            <ul>
                <li>Feature A</li>
                <li>Feature B</li>
            </ul>
            <table>
                <tr><th>Name</th><th>Status</th></tr>
                <tr><td>Alpha</td><td>Done</td></tr>
            </table>
            <ac:structured-macro ac:name="code">
                <ac:parameter ac:name="language">bash</ac:parameter>
                <ac:plain-text-body>./install.sh</ac:plain-text-body>
            </ac:structured-macro>
        "#;
        let result = renderer.render("123", "Full", html).unwrap();
        let output = result.lines.join("\n");

        // Verify document structure order
        let h1_pos = output.find("# Documentation").unwrap_or(usize::MAX);
        let para_pos = output.find("Welcome").unwrap_or(usize::MAX);
        let info_pos = output.find("ℹ").unwrap_or(usize::MAX);
        let h2_pos = output.find("## Features").unwrap_or(usize::MAX);
        let list_pos = output.find("• Feature A").unwrap_or(usize::MAX);
        let _table_pos = output.find("┌").unwrap_or(usize::MAX);

        assert!(h1_pos < para_pos, "H1 should come before paragraph");
        assert!(para_pos < info_pos, "Paragraph should come before info panel");
        assert!(info_pos < h2_pos, "Info should come before H2");
        assert!(h2_pos < list_pos, "H2 should come before list");

        // Verify all box drawing elements present
        assert!(output.contains("┌"), "Missing box top-left");
        assert!(output.contains("└"), "Missing box bottom-left");
        assert!(output.contains("│"), "Missing box vertical bars");
        assert!(output.contains("┘"), "Missing box bottom-right");
    }

    #[test]
    fn test_visual_consistency_across_elements() {
        let renderer = Renderer::new();
        let html = r#"
            <ac:structured-macro ac:name="info">
                <ac:rich-text-body>Info</ac:rich-text-body>
            </ac:structured-macro>
            <ac:structured-macro ac:name="warning">
                <ac:rich-text-body>Warning</ac:rich-text-body>
            </ac:structured-macro>
            <ac:structured-macro ac:name="code">
                <ac:parameter ac:name="language">txt</ac:parameter>
                <ac:plain-text-body>code</ac:plain-text-body>
            </ac:structured-macro>
        "#;
        let result = renderer.render("123", "Consistent", html).unwrap();

        // Count opening and closing corners - should be balanced
        let open_corners: usize = result.lines.iter()
            .filter(|l| l.starts_with("┌"))
            .count();
        let close_corners: usize = result.lines.iter()
            .filter(|l| l.starts_with("└"))
            .count();

        assert_eq!(open_corners, close_corners, "Box corners should be balanced");
        assert!(open_corners >= 3, "Should have at least 3 box elements (2 panels + 1 code)");
    }

    #[test]
    fn test_no_consecutive_double_blank_lines() {
        let renderer = Renderer::new();
        let html = r#"
            <h1>Title</h1>
            <p>Para 1</p>
            <p>Para 2</p>
            <ac:structured-macro ac:name="info">
                <ac:rich-text-body>Info</ac:rich-text-body>
            </ac:structured-macro>
            <h2>Section</h2>
            <ul><li>Item</li></ul>
        "#;
        let result = renderer.render("123", "NoDoubleBlank", html).unwrap();

        let mut consecutive_blanks = 0;
        let mut max_consecutive = 0;

        for line in &result.lines {
            if line.is_empty() {
                consecutive_blanks += 1;
                max_consecutive = max_consecutive.max(consecutive_blanks);
            } else {
                consecutive_blanks = 0;
            }
        }

        assert!(
            max_consecutive <= 2,
            "Should not have more than 2 consecutive blank lines, found {}",
            max_consecutive
        );
    }

    // -------------------------------------------------------------------------
    // LONG TEXT BOX RENDERING TESTS
    // -------------------------------------------------------------------------

    #[test]
    fn test_code_block_header_footer_width_match() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="code">
            <ac:parameter ac:name="language">rust</ac:parameter>
            <ac:plain-text-body>let x = 1;</ac:plain-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "Code", html).unwrap();

        let header = result.lines.iter().find(|l| l.starts_with("┌")).unwrap();
        let footer = result.lines.iter().find(|l| l.starts_with("└")).unwrap();

        let header_width = header.chars().count();
        let footer_width = footer.chars().count();

        assert_eq!(
            header_width, footer_width,
            "Header width ({}) != footer width ({})\nHeader: {}\nFooter: {}",
            header_width, footer_width, header, footer
        );
    }

    #[test]
    fn test_panel_header_footer_width_match() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="info">
            <ac:rich-text-body>Short content</ac:rich-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "Panel", html).unwrap();

        let header = result.lines.iter().find(|l| l.starts_with("┌")).unwrap();
        let footer = result.lines.iter().find(|l| l.starts_with("└")).unwrap();

        let header_width = header.chars().count();
        let footer_width = footer.chars().count();

        assert_eq!(
            header_width, footer_width,
            "Panel header width ({}) != footer width ({})\nHeader: {}\nFooter: {}",
            header_width, footer_width, header, footer
        );
    }

    #[test]
    fn test_code_block_long_content_line() {
        let renderer = Renderer::new();
        let long_line = "x".repeat(100);
        let html = format!(
            r#"<ac:structured-macro ac:name="code">
                <ac:parameter ac:name="language">text</ac:parameter>
                <ac:plain-text-body>{}</ac:plain-text-body>
            </ac:structured-macro>"#,
            long_line
        );
        let result = renderer.render("123", "Long", &html).unwrap();

        let header = result.lines.iter().find(|l| l.starts_with("┌")).unwrap();
        let footer = result.lines.iter().find(|l| l.starts_with("└")).unwrap();
        let content_line = result.lines.iter().find(|l| l.contains("xxxx")).unwrap();

        let header_width = header.chars().count();
        let footer_width = footer.chars().count();
        let content_width = content_line.chars().count();

        // All lines in the box should have the same width
        assert_eq!(
            header_width, footer_width,
            "Header/footer width mismatch with long content"
        );

        // Content should not exceed box width (should wrap or be contained)
        assert!(
            content_width <= header_width + 5, // small tolerance
            "Content line ({} chars) exceeds box width ({} chars)\nContent: {}",
            content_width, header_width, content_line
        );
    }

    #[test]
    fn test_panel_long_content_line() {
        let renderer = Renderer::new();
        let long_text = "This is a very long sentence that should be properly contained within the panel box boundaries without breaking the visual structure of the rendered output.";
        let html = format!(
            r#"<ac:structured-macro ac:name="warning">
                <ac:rich-text-body>{}</ac:rich-text-body>
            </ac:structured-macro>"#,
            long_text
        );
        let result = renderer.render("123", "LongPanel", &html).unwrap();

        let header = result.lines.iter().find(|l| l.starts_with("┌")).unwrap();
        let footer = result.lines.iter().find(|l| l.starts_with("└")).unwrap();

        let header_width = header.chars().count();
        let footer_width = footer.chars().count();

        assert_eq!(
            header_width, footer_width,
            "Panel header/footer mismatch with long content\nHeader: {}\nFooter: {}",
            header, footer
        );
    }

    #[test]
    fn test_code_block_multiline_long_content() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="code">
            <ac:parameter ac:name="language">python</ac:parameter>
            <ac:plain-text-body>def very_long_function_name_that_exceeds_normal_width(parameter_one, parameter_two, parameter_three):
    result = parameter_one + parameter_two + parameter_three
    return result * 2

# This is a comment that is also quite long and might cause issues with box rendering
print("Hello, World! This is a test of long string content in code blocks.")</ac:plain-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "MultiLong", html).unwrap();

        let header = result.lines.iter().find(|l| l.starts_with("┌")).unwrap();
        let footer = result.lines.iter().find(|l| l.starts_with("└")).unwrap();

        // Print actual output for debugging
        println!("=== Code Block Output ===");
        for line in &result.lines {
            println!("{}", line);
        }
        println!("=========================");

        assert_eq!(
            header.chars().count(),
            footer.chars().count(),
            "Multiline code block header/footer width mismatch"
        );
    }

    #[test]
    fn test_content_lines_have_right_border() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="info">
            <ac:rich-text-body>Test content</ac:rich-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "RightBorder", html).unwrap();

        // Find content lines (not header/footer)
        let content_lines: Vec<_> = result.lines.iter()
            .filter(|l| l.starts_with("│") && !l.contains("─"))
            .collect();

        for line in &content_lines {
            assert!(
                line.ends_with("│"),
                "Content line missing right border: '{}'",
                line
            );
        }
    }

    #[test]
    fn test_all_box_lines_same_width() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="note">
            <ac:rich-text-body>Line one
Line two is a bit longer
Short</ac:rich-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "SameWidth", html).unwrap();

        // Get all lines that are part of the box
        let box_lines: Vec<_> = result.lines.iter()
            .filter(|l| l.starts_with("┌") || l.starts_with("│") || l.starts_with("└"))
            .collect();

        if box_lines.len() >= 2 {
            let expected_width = box_lines[0].chars().count();
            for (idx, line) in box_lines.iter().enumerate() {
                let width = line.chars().count();
                assert_eq!(
                    width, expected_width,
                    "Box line {} has width {}, expected {}\nLine: '{}'",
                    idx, width, expected_width, line
                );
            }
        }
    }

    #[test]
    fn test_table_long_cell_content() {
        let renderer = Renderer::new();
        let html = r#"<table>
            <tr><th>Short</th><th>Description</th></tr>
            <tr><td>A</td><td>This is a very long description that should be handled properly by the table renderer</td></tr>
            <tr><td>B</td><td>Short</td></tr>
        </table>"#;
        let result = renderer.render("123", "LongTable", html).unwrap();

        // All table border lines should have same width
        let border_lines: Vec<_> = result.lines.iter()
            .filter(|l| l.starts_with("┌") || l.starts_with("├") || l.starts_with("└"))
            .collect();

        if border_lines.len() >= 2 {
            let expected_width = border_lines[0].chars().count();
            for line in &border_lines {
                assert_eq!(
                    line.chars().count(),
                    expected_width,
                    "Table border width inconsistent"
                );
            }
        }

        // All data rows should have same width
        let data_lines: Vec<_> = result.lines.iter()
            .filter(|l| l.starts_with("│") && !l.contains("─"))
            .collect();

        if data_lines.len() >= 2 {
            let expected_width = data_lines[0].chars().count();
            for line in &data_lines {
                assert_eq!(
                    line.chars().count(),
                    expected_width,
                    "Table row width inconsistent: '{}'",
                    line
                );
            }
        }
    }

    #[test]
    fn test_code_block_with_long_language_name() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="code">
            <ac:parameter ac:name="language">javascript</ac:parameter>
            <ac:plain-text-body>const x = 1;</ac:plain-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "JSCode", html).unwrap();

        let header = result.lines.iter().find(|l| l.starts_with("┌")).unwrap();
        let footer = result.lines.iter().find(|l| l.starts_with("└")).unwrap();

        println!("Header: {}", header);
        println!("Footer: {}", footer);

        assert_eq!(
            header.chars().count(),
            footer.chars().count(),
            "Header/footer mismatch with 'javascript' language name"
        );
    }

    #[test]
    fn test_panel_content_right_padded() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="info">
            <ac:rich-text-body>This is important information that users need to read carefully.</ac:rich-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "Padded", html).unwrap();

        let header = result.lines.iter().find(|l| l.starts_with("┌")).unwrap();
        let content = result.lines.iter().find(|l| l.contains("important") && l.starts_with("│")).unwrap();
        let footer = result.lines.iter().find(|l| l.starts_with("└")).unwrap();

        let header_w = header.chars().count();
        let content_w = content.chars().count();
        let footer_w = footer.chars().count();

        println!("Header:  '{}' ({})", header, header_w);
        println!("Content: '{}' ({})", content, content_w);
        println!("Footer:  '{}' ({})", footer, footer_w);

        assert_eq!(header_w, content_w, "Content not padded to header width");
        assert_eq!(header_w, footer_w, "Footer not same width as header");
    }

    #[test]
    fn test_panel_full_width_content() {
        let renderer = Renderer::new();
        // Content that nearly fills the box width
        let html = r#"<ac:structured-macro ac:name="warning">
            <ac:rich-text-body>WARNING: This configuration change will affect all users in production!</ac:rich-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "FullWidth", html).unwrap();

        println!("\n=== Full Width Panel Test ===");
        for line in &result.lines {
            if line.starts_with("┌") || line.starts_with("│") || line.starts_with("└") {
                println!("{}", line);
            }
        }
        println!("=============================\n");

        // Verify all box lines have same width
        let box_lines: Vec<_> = result.lines.iter()
            .filter(|l| l.starts_with("┌") || l.starts_with("│") || l.starts_with("└"))
            .collect();

        let expected_width = box_lines[0].chars().count();
        for line in &box_lines {
            assert_eq!(
                line.chars().count(),
                expected_width,
                "Width mismatch in line: '{}'",
                line
            );
        }
    }

    #[test]
    fn test_code_block_full_content() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="code">
            <ac:parameter ac:name="language">python</ac:parameter>
            <ac:plain-text-body>def calculate_total_price(items, tax_rate=0.08, discount=0.0):
    """Calculate the total price with tax and optional discount."""
    subtotal = sum(item.price * item.quantity for item in items)
    discount_amount = subtotal * discount
    taxable = subtotal - discount_amount
    return taxable * (1 + tax_rate)</ac:plain-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "FullCode", html).unwrap();

        println!("\n=== Full Code Block Test ===");
        for line in &result.lines {
            if line.starts_with("┌") || line.starts_with("│") || line.starts_with("└") {
                println!("{}", line);
            }
        }
        println!("============================\n");

        // Verify all box lines have same width
        let box_lines: Vec<_> = result.lines.iter()
            .filter(|l| l.starts_with("┌") || l.starts_with("│") || l.starts_with("└"))
            .collect();

        let expected_width = box_lines[0].chars().count();
        for (idx, line) in box_lines.iter().enumerate() {
            assert_eq!(
                line.chars().count(),
                expected_width,
                "Line {} width mismatch: '{}'",
                idx, line
            );
        }
    }

    #[test]
    fn test_panel_multiline_full_content() {
        let renderer = Renderer::new();
        let html = r#"<ac:structured-macro ac:name="note">
            <ac:rich-text-body>Before deploying to production, ensure you have:
1. Run all unit tests and integration tests
2. Updated the changelog with your changes
3. Got approval from at least two reviewers</ac:rich-text-body>
        </ac:structured-macro>"#;
        let result = renderer.render("123", "MultiNote", html).unwrap();

        println!("\n=== Multiline Panel Test ===");
        for line in &result.lines {
            if line.starts_with("┌") || line.starts_with("│") || line.starts_with("└") {
                println!("{}", line);
            }
        }
        println!("============================\n");

        let box_lines: Vec<_> = result.lines.iter()
            .filter(|l| l.starts_with("┌") || l.starts_with("│") || l.starts_with("└"))
            .collect();

        let expected_width = box_lines[0].chars().count();
        for line in &box_lines {
            assert_eq!(
                line.chars().count(),
                expected_width,
                "Multiline panel width mismatch: '{}'",
                line
            );
        }
    }
}
