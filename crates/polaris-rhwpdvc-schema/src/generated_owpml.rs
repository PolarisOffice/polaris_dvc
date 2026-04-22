//! `generated_owpml.rs` — OWPML schema model, derived from KS X 6101
//! XSDs via `tools/gen-owpml/`.
//!
//! ⚠ **Bootstrap subset**. This hand-curated initial version covers
//! the most-frequently-observed element set (roughly the top 20 % by
//! occurrence frequency across Hancom-produced HWPX samples) sufficient
//! to demonstrate the validator end-to-end against real documents.
//!
//! The proper codegen (full 455-element coverage) lands in
//! `tools/gen-owpml/` as a follow-up; this file is the interface
//! contract it targets. Replacing this module is a mechanical swap
//! once the generator runs.
//!
//! Nothing in this file is copyrighted material from the standard —
//! every entry is factual (element name, allowed children, attribute
//! name/type). The standard's *documentation prose* is not copied.

use crate::model::{AttributeDecl, ElementDecl, SchemaModel, SimpleType};

// ─────────────────────────────────────────────────────────────────────
// Shared attribute tables
// ─────────────────────────────────────────────────────────────────────

static ID_REQUIRED: &[AttributeDecl] = &[AttributeDecl {
    name: "id",
    ty: SimpleType::UnsignedInteger,
    required: true,
}];

static NO_ATTRS: &[AttributeDecl] = &[];

// ─────────────────────────────────────────────────────────────────────
// HEAD_MODEL — Contents/header.xml
// Root <hh:head> → <hh:beginNum>, <hh:refList>, …
// ─────────────────────────────────────────────────────────────────────

static HEAD_ELEMENTS: &[(&str, ElementDecl)] = &[
    (
        "head",
        ElementDecl {
            name: "head",
            children: &[
                ("beginNum", 0, Some(1)),
                ("refList", 1, Some(1)),
                ("forbiddenWordList", 0, Some(1)),
                ("compatibleDocument", 0, Some(1)),
                ("trackchangeConfig", 0, Some(1)),
                ("trackChangeEncrpytion", 0, Some(1)),
                ("docOption", 0, Some(1)),
                ("metaTag", 0, Some(1)),
            ],
            attributes: &[
                AttributeDecl {
                    name: "version",
                    ty: SimpleType::String,
                    required: false,
                },
                AttributeDecl {
                    name: "secCnt",
                    ty: SimpleType::UnsignedInteger,
                    required: false,
                },
            ],
            text_allowed: false,
        },
    ),
    (
        "beginNum",
        ElementDecl {
            name: "beginNum",
            children: &[],
            attributes: &[
                AttributeDecl {
                    name: "page",
                    ty: SimpleType::UnsignedInteger,
                    required: false,
                },
                AttributeDecl {
                    name: "footnote",
                    ty: SimpleType::UnsignedInteger,
                    required: false,
                },
                AttributeDecl {
                    name: "endnote",
                    ty: SimpleType::UnsignedInteger,
                    required: false,
                },
                AttributeDecl {
                    name: "pic",
                    ty: SimpleType::UnsignedInteger,
                    required: false,
                },
                AttributeDecl {
                    name: "tbl",
                    ty: SimpleType::UnsignedInteger,
                    required: false,
                },
                AttributeDecl {
                    name: "equation",
                    ty: SimpleType::UnsignedInteger,
                    required: false,
                },
            ],
            text_allowed: false,
        },
    ),
    (
        "refList",
        ElementDecl {
            name: "refList",
            children: &[
                ("fontfaces", 0, Some(1)),
                ("borderFills", 0, Some(1)),
                ("charProperties", 0, Some(1)),
                ("paraProperties", 0, Some(1)),
                ("styles", 0, Some(1)),
                ("numberings", 0, Some(1)),
                ("bullets", 0, Some(1)),
                ("memoProperties", 0, Some(1)),
                ("trackChanges", 0, Some(1)),
                ("trackChangeAuthors", 0, Some(1)),
            ],
            attributes: NO_ATTRS,
            text_allowed: false,
        },
    ),
    (
        "fontfaces",
        ElementDecl {
            name: "fontfaces",
            children: &[("fontface", 1, None)],
            attributes: &[AttributeDecl {
                name: "itemCnt",
                ty: SimpleType::UnsignedInteger,
                required: false,
            }],
            text_allowed: false,
        },
    ),
    (
        "fontface",
        ElementDecl {
            name: "fontface",
            children: &[("font", 1, None)],
            attributes: &[
                AttributeDecl {
                    name: "lang",
                    ty: SimpleType::Enum(&[
                        "HANGUL", "LATIN", "HANJA", "JAPANESE", "OTHER", "SYMBOL", "USER",
                    ]),
                    required: true,
                },
                AttributeDecl {
                    name: "fontCnt",
                    ty: SimpleType::UnsignedInteger,
                    required: false,
                },
            ],
            text_allowed: false,
        },
    ),
    (
        "font",
        ElementDecl {
            name: "font",
            children: &[("typeInfo", 0, Some(1)), ("substFont", 0, Some(1))],
            attributes: &[
                AttributeDecl {
                    name: "id",
                    ty: SimpleType::UnsignedInteger,
                    required: true,
                },
                AttributeDecl {
                    name: "face",
                    ty: SimpleType::String,
                    required: true,
                },
                AttributeDecl {
                    name: "type",
                    ty: SimpleType::Enum(&["TTF", "HFT", "RTTF", "UNKNOWN"]),
                    required: false,
                },
                AttributeDecl {
                    name: "isEmbedded",
                    ty: SimpleType::Boolean,
                    required: false,
                },
            ],
            text_allowed: false,
        },
    ),
    (
        "charProperties",
        ElementDecl {
            name: "charProperties",
            children: &[("charPr", 1, None)],
            attributes: &[AttributeDecl {
                name: "itemCnt",
                ty: SimpleType::UnsignedInteger,
                required: false,
            }],
            text_allowed: false,
        },
    ),
    (
        "charPr",
        ElementDecl {
            name: "charPr",
            children: &[
                ("fontRef", 1, Some(1)),
                ("ratio", 0, Some(1)),
                ("spacing", 0, Some(1)),
                ("relSz", 0, Some(1)),
                ("offset", 0, Some(1)),
                ("italic", 0, Some(1)),
                ("bold", 0, Some(1)),
                ("underline", 0, Some(1)),
                ("strikeout", 0, Some(1)),
                ("outline", 0, Some(1)),
                ("shadow", 0, Some(1)),
                ("emboss", 0, Some(1)),
                ("engrave", 0, Some(1)),
                ("supscript", 0, Some(1)),
                ("subscript", 0, Some(1)),
                ("symMark", 0, Some(1)),
                ("border", 0, Some(1)),
            ],
            attributes: &[
                AttributeDecl {
                    name: "id",
                    ty: SimpleType::UnsignedInteger,
                    required: true,
                },
                AttributeDecl {
                    name: "height",
                    ty: SimpleType::UnsignedInteger,
                    required: false,
                },
                AttributeDecl {
                    name: "textColor",
                    ty: SimpleType::String,
                    required: false,
                },
                AttributeDecl {
                    name: "shadeColor",
                    ty: SimpleType::String,
                    required: false,
                },
                AttributeDecl {
                    name: "useFontSpace",
                    ty: SimpleType::Boolean,
                    required: false,
                },
                AttributeDecl {
                    name: "useKerning",
                    ty: SimpleType::Boolean,
                    required: false,
                },
                AttributeDecl {
                    name: "symMark",
                    ty: SimpleType::String,
                    required: false,
                },
                AttributeDecl {
                    name: "borderFillIDRef",
                    ty: SimpleType::Reference,
                    required: false,
                },
            ],
            text_allowed: false,
        },
    ),
    (
        "fontRef",
        ElementDecl {
            name: "fontRef",
            children: &[],
            attributes: &[
                AttributeDecl {
                    name: "hangul",
                    ty: SimpleType::Reference,
                    required: false,
                },
                AttributeDecl {
                    name: "latin",
                    ty: SimpleType::Reference,
                    required: false,
                },
                AttributeDecl {
                    name: "hanja",
                    ty: SimpleType::Reference,
                    required: false,
                },
                AttributeDecl {
                    name: "japanese",
                    ty: SimpleType::Reference,
                    required: false,
                },
                AttributeDecl {
                    name: "other",
                    ty: SimpleType::Reference,
                    required: false,
                },
                AttributeDecl {
                    name: "symbol",
                    ty: SimpleType::Reference,
                    required: false,
                },
                AttributeDecl {
                    name: "user",
                    ty: SimpleType::Reference,
                    required: false,
                },
            ],
            text_allowed: false,
        },
    ),
    (
        "paraProperties",
        ElementDecl {
            name: "paraProperties",
            children: &[("paraPr", 1, None)],
            attributes: &[AttributeDecl {
                name: "itemCnt",
                ty: SimpleType::UnsignedInteger,
                required: false,
            }],
            text_allowed: false,
        },
    ),
    (
        "paraPr",
        ElementDecl {
            name: "paraPr",
            children: &[
                ("align", 0, Some(1)),
                ("heading", 0, Some(1)),
                ("breakSetting", 0, Some(1)),
                ("margin", 0, Some(1)),
                ("lineSpacing", 0, Some(1)),
                ("border", 0, Some(1)),
                ("autoSpacing", 0, Some(1)),
                ("tabPr", 0, Some(1)),
            ],
            attributes: &[
                AttributeDecl {
                    name: "id",
                    ty: SimpleType::UnsignedInteger,
                    required: true,
                },
                AttributeDecl {
                    name: "tabPrIDRef",
                    ty: SimpleType::Reference,
                    required: false,
                },
                AttributeDecl {
                    name: "condense",
                    ty: SimpleType::UnsignedInteger,
                    required: false,
                },
                AttributeDecl {
                    name: "fontLineHeight",
                    ty: SimpleType::Boolean,
                    required: false,
                },
                AttributeDecl {
                    name: "snapToGrid",
                    ty: SimpleType::Boolean,
                    required: false,
                },
                AttributeDecl {
                    name: "suppressLineNumbers",
                    ty: SimpleType::Boolean,
                    required: false,
                },
                AttributeDecl {
                    name: "checked",
                    ty: SimpleType::Boolean,
                    required: false,
                },
            ],
            text_allowed: false,
        },
    ),
    (
        "styles",
        ElementDecl {
            name: "styles",
            children: &[("style", 0, None)],
            attributes: &[AttributeDecl {
                name: "itemCnt",
                ty: SimpleType::UnsignedInteger,
                required: false,
            }],
            text_allowed: false,
        },
    ),
    (
        "style",
        ElementDecl {
            name: "style",
            children: &[],
            attributes: &[
                AttributeDecl {
                    name: "id",
                    ty: SimpleType::UnsignedInteger,
                    required: true,
                },
                AttributeDecl {
                    name: "type",
                    ty: SimpleType::String,
                    required: false,
                },
                AttributeDecl {
                    name: "name",
                    ty: SimpleType::String,
                    required: false,
                },
                AttributeDecl {
                    name: "engName",
                    ty: SimpleType::String,
                    required: false,
                },
                AttributeDecl {
                    name: "paraPrIDRef",
                    ty: SimpleType::Reference,
                    required: false,
                },
                AttributeDecl {
                    name: "charPrIDRef",
                    ty: SimpleType::Reference,
                    required: false,
                },
                AttributeDecl {
                    name: "nextStyleIDRef",
                    ty: SimpleType::Reference,
                    required: false,
                },
                AttributeDecl {
                    name: "langID",
                    ty: SimpleType::UnsignedInteger,
                    required: false,
                },
                AttributeDecl {
                    name: "lockForm",
                    ty: SimpleType::Boolean,
                    required: false,
                },
            ],
            text_allowed: false,
        },
    ),
];

pub static HEAD_MODEL: SchemaModel = SchemaModel {
    root_name: "head",
    elements: HEAD_ELEMENTS,
};

// ─────────────────────────────────────────────────────────────────────
// SECTION_MODEL — Contents/section*.xml
// Root <hs:sec> → <hp:p> → <hp:run> → <hp:t>, etc.
// ─────────────────────────────────────────────────────────────────────

static SECTION_ELEMENTS: &[(&str, ElementDecl)] = &[
    (
        "sec",
        ElementDecl {
            name: "sec",
            children: &[("p", 0, None)],
            attributes: NO_ATTRS,
            text_allowed: false,
        },
    ),
    (
        "p",
        ElementDecl {
            name: "p",
            children: &[("run", 0, None), ("linesegarray", 0, Some(1))],
            attributes: &[
                AttributeDecl {
                    name: "id",
                    ty: SimpleType::UnsignedInteger,
                    required: false,
                },
                AttributeDecl {
                    name: "paraPrIDRef",
                    ty: SimpleType::Reference,
                    required: false,
                },
                AttributeDecl {
                    name: "styleIDRef",
                    ty: SimpleType::Reference,
                    required: false,
                },
                AttributeDecl {
                    name: "pageBreak",
                    ty: SimpleType::Boolean,
                    required: false,
                },
                AttributeDecl {
                    name: "columnBreak",
                    ty: SimpleType::Boolean,
                    required: false,
                },
                AttributeDecl {
                    name: "merged",
                    ty: SimpleType::Boolean,
                    required: false,
                },
            ],
            text_allowed: false,
        },
    ),
    (
        "run",
        ElementDecl {
            name: "run",
            children: &[
                ("t", 0, None),
                ("ctrl", 0, None),
                ("tbl", 0, None),
                ("rect", 0, None),
                ("line", 0, None),
                ("ellipse", 0, None),
                ("arc", 0, None),
                ("polygon", 0, None),
                ("curve", 0, None),
                ("connectLine", 0, None),
                ("pic", 0, None),
                ("ole", 0, None),
                ("container", 0, None),
                ("equation", 0, None),
                ("formObject", 0, None),
                ("fieldBegin", 0, None),
                ("fieldEnd", 0, None),
                ("secPr", 0, Some(1)),
            ],
            attributes: &[AttributeDecl {
                name: "charPrIDRef",
                ty: SimpleType::Reference,
                required: false,
            }],
            text_allowed: false,
        },
    ),
    (
        "t",
        ElementDecl {
            name: "t",
            children: &[
                ("markpenBegin", 0, None),
                ("markpenEnd", 0, None),
                ("titleMark", 0, None),
                ("tab", 0, None),
                ("lineBreak", 0, None),
                ("hyphen", 0, None),
                ("nbSpace", 0, None),
                ("fwSpace", 0, None),
                ("insertBegin", 0, None),
                ("insertEnd", 0, None),
                ("deleteBegin", 0, None),
                ("deleteEnd", 0, None),
            ],
            attributes: &[
                AttributeDecl {
                    name: "charPrIDRef",
                    ty: SimpleType::Reference,
                    required: false,
                },
                AttributeDecl {
                    name: "lang",
                    ty: SimpleType::String,
                    required: false,
                },
            ],
            text_allowed: true,
        },
    ),
    (
        "linesegarray",
        ElementDecl {
            name: "linesegarray",
            children: &[("lineseg", 1, None)],
            attributes: NO_ATTRS,
            text_allowed: false,
        },
    ),
    (
        "lineseg",
        ElementDecl {
            name: "lineseg",
            children: &[],
            attributes: &[
                AttributeDecl {
                    name: "textpos",
                    ty: SimpleType::UnsignedInteger,
                    required: false,
                },
                AttributeDecl {
                    name: "vertpos",
                    ty: SimpleType::Integer,
                    required: false,
                },
                AttributeDecl {
                    name: "vertsize",
                    ty: SimpleType::UnsignedInteger,
                    required: false,
                },
                AttributeDecl {
                    name: "textheight",
                    ty: SimpleType::UnsignedInteger,
                    required: false,
                },
                AttributeDecl {
                    name: "baseline",
                    ty: SimpleType::UnsignedInteger,
                    required: false,
                },
                AttributeDecl {
                    name: "spacelet",
                    ty: SimpleType::UnsignedInteger,
                    required: false,
                },
                AttributeDecl {
                    name: "indent",
                    ty: SimpleType::Integer,
                    required: false,
                },
                AttributeDecl {
                    name: "horzpos",
                    ty: SimpleType::Integer,
                    required: false,
                },
                AttributeDecl {
                    name: "horzsize",
                    ty: SimpleType::UnsignedInteger,
                    required: false,
                },
                AttributeDecl {
                    name: "flags",
                    ty: SimpleType::String,
                    required: false,
                },
            ],
            text_allowed: false,
        },
    ),
];

pub static SECTION_MODEL: SchemaModel = SchemaModel {
    root_name: "sec",
    elements: SECTION_ELEMENTS,
};

// ─────────────────────────────────────────────────────────────────────
// CONTENT_HPF_MODEL — Contents/content.hpf (OPF package)
// Small bundled schema — OPF isn't part of KS X 6101 proper.
// ─────────────────────────────────────────────────────────────────────

static CONTENT_HPF_ELEMENTS: &[(&str, ElementDecl)] = &[
    (
        "package",
        ElementDecl {
            name: "package",
            children: &[
                ("metadata", 0, Some(1)),
                ("manifest", 1, Some(1)),
                ("spine", 0, Some(1)),
            ],
            attributes: &[AttributeDecl {
                name: "version",
                ty: SimpleType::String,
                required: false,
            }],
            text_allowed: false,
        },
    ),
    (
        "manifest",
        ElementDecl {
            name: "manifest",
            children: &[("item", 1, None)],
            attributes: NO_ATTRS,
            text_allowed: false,
        },
    ),
    (
        "item",
        ElementDecl {
            name: "item",
            children: &[],
            attributes: &[
                AttributeDecl {
                    name: "id",
                    ty: SimpleType::String,
                    required: true,
                },
                AttributeDecl {
                    name: "href",
                    ty: SimpleType::String,
                    required: true,
                },
                AttributeDecl {
                    name: "media-type",
                    ty: SimpleType::String,
                    required: false,
                },
            ],
            text_allowed: false,
        },
    ),
    (
        "spine",
        ElementDecl {
            name: "spine",
            children: &[("itemref", 0, None)],
            attributes: NO_ATTRS,
            text_allowed: false,
        },
    ),
    (
        "itemref",
        ElementDecl {
            name: "itemref",
            children: &[],
            attributes: &[AttributeDecl {
                name: "idref",
                ty: SimpleType::String,
                required: true,
            }],
            text_allowed: false,
        },
    ),
];

pub static CONTENT_HPF_MODEL: SchemaModel = SchemaModel {
    root_name: "package",
    elements: CONTENT_HPF_ELEMENTS,
};

// ─────────────────────────────────────────────────────────────────────
// SETTINGS_MODEL / VERSION_MODEL — thin stubs for now. The full KS X
// 6101 schemas contain rich content for these; codegen will populate.
// ─────────────────────────────────────────────────────────────────────

static SETTINGS_ELEMENTS: &[(&str, ElementDecl)] = &[(
    "HWPApplicationSetting",
    ElementDecl {
        name: "HWPApplicationSetting",
        children: &[],
        attributes: NO_ATTRS,
        text_allowed: true,
    },
)];

pub static SETTINGS_MODEL: SchemaModel = SchemaModel {
    root_name: "HWPApplicationSetting",
    elements: SETTINGS_ELEMENTS,
};

static VERSION_ELEMENTS: &[(&str, ElementDecl)] = &[(
    "HCFVersion",
    ElementDecl {
        name: "HCFVersion",
        children: &[],
        attributes: &[
            AttributeDecl {
                name: "targetApplication",
                ty: SimpleType::String,
                required: false,
            },
            AttributeDecl {
                name: "major",
                ty: SimpleType::UnsignedInteger,
                required: false,
            },
            AttributeDecl {
                name: "minor",
                ty: SimpleType::UnsignedInteger,
                required: false,
            },
            AttributeDecl {
                name: "micro",
                ty: SimpleType::UnsignedInteger,
                required: false,
            },
            AttributeDecl {
                name: "buildNumber",
                ty: SimpleType::UnsignedInteger,
                required: false,
            },
        ],
        text_allowed: true,
    },
)];

pub static VERSION_MODEL: SchemaModel = SchemaModel {
    root_name: "HCFVersion",
    elements: VERSION_ELEMENTS,
};

// Silence unused-warning for the shared helpers when no root touches them.
#[allow(dead_code)]
static _KEEP_ALIVE: &[&[AttributeDecl]] = &[ID_REQUIRED];
