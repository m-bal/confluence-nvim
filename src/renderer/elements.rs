//! Rendering for specific Confluence elements

/// Code block representation
#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub language: String,
    pub code: String,
    pub line_numbers: bool,
}

/// Macro panel (info, warning, error, etc.)
#[derive(Debug, Clone)]
pub struct MacroPanel {
    pub macro_type: MacroType,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MacroType {
    Info,
    Warning,
    Error,
    Note,
    Success,
}

/// Table representation
#[derive(Debug, Clone)]
pub struct Table {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl CodeBlock {
    /// Render code block with box-drawing characters
    pub fn render(&self) -> Vec<String> {
        let mut lines = Vec::new();

        // Header
        let header = format!("┌─ {} {}", self.language.to_uppercase(), "─".repeat(50));
        lines.push(header);

        // Code content
        for (idx, line) in self.code.lines().enumerate() {
            if self.line_numbers {
                lines.push(format!("│ {:3}  {}", idx + 1, line));
            } else {
                lines.push(format!("│ {}", line));
            }
        }

        // Footer
        lines.push("└─────────────────────────────────────────────────┘".to_string());

        lines
    }
}

impl MacroPanel {
    /// Get the icon for this macro type
    pub fn icon(&self) -> &'static str {
        match self.macro_type {
            MacroType::Info => "ℹ",
            MacroType::Warning => "⚠",
            MacroType::Error => "✗",
            MacroType::Note => "📝",
            MacroType::Success => "✓",
        }
    }

    /// Get the label for this macro type
    pub fn label(&self) -> &'static str {
        match self.macro_type {
            MacroType::Info => "INFO",
            MacroType::Warning => "WARNING",
            MacroType::Error => "ERROR",
            MacroType::Note => "NOTE",
            MacroType::Success => "SUCCESS",
        }
    }

    /// Render macro panel with box-drawing characters
    pub fn render(&self) -> Vec<String> {
        let mut lines = Vec::new();

        // Header with icon and label
        let header = format!("┌─ {} {} {}", self.icon(), self.label(), "─".repeat(40));
        lines.push(header);

        // Content
        for line in self.content.lines() {
            lines.push(format!("│ {}", line));
        }

        // Footer
        lines.push("└─────────────────────────────────────────────────┘".to_string());

        lines
    }
}

impl Table {
    /// Render table with box-drawing characters
    pub fn render(&self) -> Vec<String> {
        let mut lines = Vec::new();

        // Calculate column widths
        let col_widths = self.calculate_column_widths();

        // Top border
        let top = format!("┌{}┐",
            col_widths.iter()
                .map(|w| "─".repeat(w + 2))
                .collect::<Vec<_>>()
                .join("┬")
        );
        lines.push(top);

        // Header row
        let header = self.render_row(&self.headers, &col_widths);
        lines.push(header);

        // Header separator
        let separator = format!("├{}┤",
            col_widths.iter()
                .map(|w| "─".repeat(w + 2))
                .collect::<Vec<_>>()
                .join("┼")
        );
        lines.push(separator);

        // Data rows
        for row in &self.rows {
            lines.push(self.render_row(row, &col_widths));
        }

        // Bottom border
        let bottom = format!("└{}┘",
            col_widths.iter()
                .map(|w| "─".repeat(w + 2))
                .collect::<Vec<_>>()
                .join("┴")
        );
        lines.push(bottom);

        lines
    }

    fn calculate_column_widths(&self) -> Vec<usize> {
        let mut widths = self.headers.iter().map(|h| h.len()).collect::<Vec<_>>();

        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        widths
    }

    fn render_row(&self, cells: &[String], widths: &[usize]) -> String {
        let padded_cells: Vec<String> = cells.iter()
            .zip(widths.iter())
            .map(|(cell, width)| format!(" {:<width$} ", cell, width = width))
            .collect();

        format!("│{}│", padded_cells.join("│"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_block_render() {
        let block = CodeBlock {
            language: "rust".to_string(),
            code: "fn main() {\n    println!(\"Hello\");\n}".to_string(),
            line_numbers: true,
        };

        let lines = block.render();
        assert!(lines[0].contains("RUST"));
        assert!(lines.len() > 3); // Header + content + footer
    }

    #[test]
    fn test_macro_panel_icons() {
        let info = MacroPanel {
            macro_type: MacroType::Info,
            content: "Test".to_string(),
        };
        assert_eq!(info.icon(), "ℹ");
        assert_eq!(info.label(), "INFO");
    }

    #[test]
    fn test_table_render() {
        let table = Table {
            headers: vec!["Name".to_string(), "Status".to_string()],
            rows: vec![
                vec!["Feature A".to_string(), "Done".to_string()],
                vec!["Feature B".to_string(), "In Progress".to_string()],
            ],
        };

        let lines = table.render();
        assert!(lines[0].starts_with("┌"));
        assert!(lines.last().unwrap().starts_with("└"));
    }
}
