use ratatui::prelude::Color;



// ========= 非专有 ===========

/// 白色（非最白）
pub const CL_WHITE: Color = Color::from_u32(0xDADADA);
/// 白色（相比CL_WHITE 更黑一些）
pub const CL_DARK_WHITE: Color = Color::from_u32(0xC6C6C6);
/// 白色（相比CL_DARK_WHITE更黑一些, 类似灰...）
pub const CL_DARK_DARK_WHITE: Color =Color::from_u32(0x6E6E6E);
/// 黑色 （非最黑）
pub const CL_BLACK: Color = Color::from_u32(0x252525);
/// 黑色 （相比CL_BLACK 更白一些
pub const CL_LIGHT_BLACK: Color = Color::from_u32(0x454545);
/// 黑色 （相比C_LIGHT_BLACK更白一些，类似灰...)
pub const CL_LIGHT_LIGHT_BLACK: Color = Color::from_u32(0x555555);
/// 红色
pub const CL_RED: Color = Color::Red;
/// 深红
pub const CL_DARK_RED: Color = Color::from_u32(0x6F0000);
/// 深深红
pub const CL_DARK_DARK_RED: Color = Color::from_u32(0x1A0000);
/// 深深深红
pub const CL_DARK_DARK_DARK_RED: Color = Color::from_u32(0x110000);

// ========= 专有 ==========

/// 深灰色背景
pub const CL_GLOBAL_BG: Color = CL_BLACK;
/// title color
pub const CL_GLOBAL_TITLE: Color = CL_LIGHT_BLACK;





