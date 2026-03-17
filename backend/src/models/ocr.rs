use serde::{Deserialize, Serialize};

/// Structured product design information extracted by Tesseract OCR
/// and parsed by Claude.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrAnalysis {
    /// Raw text output from Tesseract before any structuring.
    pub raw_text:      String,
    /// Text strings detected on the product surface.
    pub detected_text: Vec<String>,
    /// Apparent font style (e.g. "script", "sans", "handwritten").
    pub font_hint:     String,
    /// Dominant colour palette description.
    pub color_hint:    String,
    /// Product category (mug, tshirt, poster, etc.).
    pub product_type:  String,
    /// Two-sentence design summary suitable for DALL-E prompt generation.
    pub summary:       String,
}
