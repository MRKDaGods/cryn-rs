use egui::{Color32, FontId, Ui};

pub fn get_trunacted_text(
    ui: &mut Ui,
    text: &str,
    font_size: f32,
    width: f32,
    max_lines: usize,
) -> String {
    let rows = &ui
        .painter()
        .layout(
            text.to_owned(),
            FontId::proportional(font_size),
            Color32::PLACEHOLDER,
            width,
        )
        .rows;

    // Truncate and add ellipsis if needed
    if rows.len() > max_lines {
        let mut text = String::new();

        for row in rows.iter().take(max_lines) {
            text.push_str(&row.text());
        }

        // Remove last 3 chars and add ellipsis
        text = text.chars().take(text.len().saturating_sub(3)).collect();
        text.push_str("...");

        return text;
    }

    text.to_owned()
}
