//! Neovim highlight groups for Confluence content

/// Highlight group definition
#[derive(Debug, Clone)]
pub struct HighlightGroup {
    /// Line number (0-indexed)
    pub line: usize,

    /// Column start (0-indexed)
    pub col_start: usize,

    /// Column end (0-indexed, exclusive)
    pub col_end: usize,

    /// Highlight group name
    pub group: String,
}

/// Confluence-specific highlight groups
pub struct ConfluenceHighlights;

impl ConfluenceHighlights {
    /// Info panel highlight
    pub const INFO_PANEL: &'static str = "ConfluenceInfoPanel";

    /// Warning panel highlight
    pub const WARNING_PANEL: &'static str = "ConfluenceWarningPanel";

    /// Error panel highlight
    pub const ERROR_PANEL: &'static str = "ConfluenceErrorPanel";

    /// Success panel highlight
    pub const SUCCESS_PANEL: &'static str = "ConfluenceSuccessPanel";

    /// Note panel highlight
    pub const NOTE_PANEL: &'static str = "ConfluenceNotePanel";

    /// Code block highlight
    pub const CODE_BLOCK: &'static str = "ConfluenceCodeBlock";

    /// Table header highlight
    pub const TABLE_HEADER: &'static str = "ConfluenceTableHeader";

    /// Link highlight
    pub const LINK: &'static str = "ConfluenceLink";

    /// Comment indicator highlight
    pub const COMMENT: &'static str = "ConfluenceComment";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_group_creation() {
        let hl = HighlightGroup {
            line: 5,
            col_start: 0,
            col_end: 10,
            group: ConfluenceHighlights::INFO_PANEL.to_string(),
        };

        assert_eq!(hl.line, 5);
        assert_eq!(hl.group, "ConfluenceInfoPanel");
    }
}
