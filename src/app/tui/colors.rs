//! 各颜色常量
//!
//! 以 CL 开头，`D` 前缀为更深色一些的，`L` 前缀为更浅色一些的

use ratatui::prelude::Color;

pub const CL_WHITE: Color = Color::from_u32(0xDADADA);

pub const CL_D_WHITE: Color = Color::from_u32(0xC6C6C6);

pub const CL_DD_WHITE: Color = Color::from_u32(0xAAAAAA);

pub const CL_DDD_WHITE: Color = Color::from_u32(0x6E6E6E);

pub const CL_BLACK: Color = Color::from_u32(0x202020);

pub const CL_L_BLACK: Color = Color::from_u32(0x303030);

pub const CL_LL_BLACK: Color = Color::from_u32(0x404040);

#[cfg(test)]
pub const CL_LLL_BLACK: Color = Color::from_u32(0x555555);

pub const CL_RED: Color = Color::Red;

pub const CL_D_RED: Color = Color::from_u32(0x6F0000);

pub const CL_DD_RED: Color = Color::from_u32(0x1A0000);

pub const CL_DDD_RED: Color = Color::from_u32(0x110000);
// 极霸微软黄
pub const CL_YELLOW: Color = Color::from_u32(0xFFCC00);

pub const CL_D_YELLOW: Color = Color::from_u32(0xCCA500);

pub const CL_BLUE: Color = Color::from_u32(0x0099CC);

// 安卡颜色
pub const CL_AK: Color = Color::from_u32(0xD3F037);
