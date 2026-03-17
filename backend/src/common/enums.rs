use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported e-commerce platforms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Etsy,
    Amazon,
}

impl Platform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::Etsy   => "etsy",
            Platform::Amazon => "amazon",
        }
    }

    pub fn photography_guidelines(&self) -> &'static str {
        match self {
            Platform::Etsy =>
                "Etsy: handmade, artisanal, lifestyle and gifting aesthetics. \
                 Warm tones, story-driven scenes, cosy atmosphere.",
            Platform::Amazon =>
                "Amazon: pure white background for main image. \
                 High contrast, clear product visibility. Bold lifestyle for alternate images.",
        }
    }

    pub fn qa_criteria(&self) -> &'static str {
        match self {
            Platform::Etsy   => "artisanal quality, warm lifestyle feel, gifting appeal",
            Platform::Amazon => "clear product visibility, professional look, clean background",
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<&str> for Platform {
    type Error = crate::common::error::AppError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "etsy"   => Ok(Platform::Etsy),
            "amazon" => Ok(Platform::Amazon),
            other    => Err(crate::common::error::AppError::UnsupportedPlatform(
                format!("'{}' is not a supported platform. Valid values: etsy, amazon", other)
            )),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────

/// The 4 fixed condition slots every product run generates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConditionSlot {
    C1, // Daily Use × White Studio
    C2, // Gift Presentation × Warm Lifestyle
    C3, // Professional × Dark Dramatic
    C4, // Outdoor/Adventure × Natural Outdoor
}

impl ConditionSlot {
    pub fn id(&self) -> &'static str {
        match self {
            ConditionSlot::C1 => "c1",
            ConditionSlot::C2 => "c2",
            ConditionSlot::C3 => "c3",
            ConditionSlot::C4 => "c4",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ConditionSlot::C1 => "Daily Use",
            ConditionSlot::C2 => "Gift Presentation",
            ConditionSlot::C3 => "Professional",
            ConditionSlot::C4 => "Outdoor / Adventure",
        }
    }

    pub fn environment(&self) -> &'static str {
        match self {
            ConditionSlot::C1 => "White Studio",
            ConditionSlot::C2 => "Warm Lifestyle",
            ConditionSlot::C3 => "Dark Dramatic",
            ConditionSlot::C4 => "Natural Outdoor",
        }
    }

    pub fn aesthetic_guidance(&self) -> &'static str {
        match self {
            ConditionSlot::C1 => "clean, minimal, bright, everyday use",
            ConditionSlot::C2 => "cozy, warm bokeh, gift-ready, artisan",
            ConditionSlot::C3 => "moody, editorial, dark background, premium",
            ConditionSlot::C4 => "earthy, natural light, organic, nature-inspired",
        }
    }

    pub fn all() -> [ConditionSlot; 4] {
        [ConditionSlot::C1, ConditionSlot::C2, ConditionSlot::C3, ConditionSlot::C4]
    }
}

impl fmt::Display for ConditionSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} × {}", self.label(), self.environment())
    }
}

// ─────────────────────────────────────────────────────────────────────────────

/// Status of a pipeline stage execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StageStatus {
    Ok,
    Skipped,
    Error,
}

impl fmt::Display for StageStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            StageStatus::Ok      => "ok",
            StageStatus::Skipped => "skipped",
            StageStatus::Error   => "error",
        };
        write!(f, "{}", s)
    }
}

// ─────────────────────────────────────────────────────────────────────────────

/// Product types detectable by OCR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductType {
    Mug,
    Tshirt,
    Poster,
    Tote,
    Candle,
    PhoneCase,
    Pillow,
    Print,
    Other,
}

impl fmt::Display for ProductType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ProductType::Mug        => "mug",
            ProductType::Tshirt     => "tshirt",
            ProductType::Poster     => "poster",
            ProductType::Tote       => "tote",
            ProductType::Candle     => "candle",
            ProductType::PhoneCase  => "phone_case",
            ProductType::Pillow     => "pillow",
            ProductType::Print      => "print",
            ProductType::Other      => "other",
        };
        write!(f, "{}", s)
    }
}

// ─────────────────────────────────────────────────────────────────────────────

/// DALL-E 3 output size options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageSize {
    Square,    // 1024×1024
    Landscape, // 1792×1024
    Portrait,  // 1024×1792
}

impl ImageSize {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImageSize::Square    => "1024x1024",
            ImageSize::Landscape => "1792x1024",
            ImageSize::Portrait  => "1024x1792",
        }
    }
}

impl Default for ImageSize {
    fn default() -> Self { ImageSize::Square }
}

// ─────────────────────────────────────────────────────────────────────────────

/// DALL-E 3 quality tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageQuality {
    Standard,
    Hd,
}

impl ImageQuality {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImageQuality::Standard => "standard",
            ImageQuality::Hd       => "hd",
        }
    }
}

impl Default for ImageQuality {
    fn default() -> Self { ImageQuality::Standard }
}
