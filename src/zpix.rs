
// GENERATED CODE by convert-bdf in tools
//
// it only output 3 parts: s_glyphs, s_data, and final FONT_ZPIX.
// You maybe reorganize according to your needs. For example, put the s_data into eeprom, 
// write you code that read it from eeprom, build a BdfFont instance with reference to FONT_ZPIX and delete FONT_ZPIX. 
//
pub use  unformatted::FONT_ZPIX;
#[rustfmt::skip]
mod unformatted {
    use embedded_fonts::{BdfGlyph,BdfFont};
    use embedded_graphics::{
        prelude::*,
        primitives::Rectangle,
    };

    const s_glyphs:[BdfGlyph;22] = [BdfGlyph { character: '.', bounding_box: Rectangle { top_left: Point { x: 1, y: 0 }, size: Size { width: 1, height: 2 } }, device_width: 4, start_index: 0 }, BdfGlyph { character: '0', bounding_box: Rectangle { top_left: Point { x: 0, y: -7 }, size: Size { width: 5, height: 9 } }, device_width: 7, start_index: 2 }, BdfGlyph { character: '1', bounding_box: Rectangle { top_left: Point { x: 0, y: -7 }, size: Size { width: 2, height: 9 } }, device_width: 4, start_index: 47 }, BdfGlyph { character: '2', bounding_box: Rectangle { top_left: Point { x: 0, y: -7 }, size: Size { width: 5, height: 9 } }, device_width: 7, start_index: 65 }, BdfGlyph { character: '3', bounding_box: Rectangle { top_left: Point { x: 0, y: -7 }, size: Size { width: 5, height: 9 } }, device_width: 7, start_index: 110 }, BdfGlyph { character: '4', bounding_box: Rectangle { top_left: Point { x: 0, y: -7 }, size: Size { width: 5, height: 9 } }, device_width: 7, start_index: 155 }, BdfGlyph { character: '5', bounding_box: Rectangle { top_left: Point { x: 0, y: -7 }, size: Size { width: 5, height: 9 } }, device_width: 7, start_index: 200 }, BdfGlyph { character: '6', bounding_box: Rectangle { top_left: Point { x: 0, y: -7 }, size: Size { width: 5, height: 9 } }, device_width: 7, start_index: 245 }, BdfGlyph { character: '7', bounding_box: Rectangle { top_left: Point { x: 0, y: -7 }, size: Size { width: 5, height: 9 } }, device_width: 7, start_index: 290 }, BdfGlyph { character: '8', bounding_box: Rectangle { top_left: Point { x: 0, y: -7 }, size: Size { width: 5, height: 9 } }, device_width: 7, start_index: 335 }, BdfGlyph { character: '9', bounding_box: Rectangle { top_left: Point { x: 0, y: -7 }, size: Size { width: 5, height: 9 } }, device_width: 7, start_index: 380 }, BdfGlyph { character: 'B', bounding_box: Rectangle { top_left: Point { x: 0, y: -7 }, size: Size { width: 5, height: 9 } }, device_width: 7, start_index: 425 }, BdfGlyph { character: 'G', bounding_box: Rectangle { top_left: Point { x: 0, y: -7 }, size: Size { width: 5, height: 9 } }, device_width: 7, start_index: 470 }, BdfGlyph { character: 'K', bounding_box: Rectangle { top_left: Point { x: 0, y: -7 }, size: Size { width: 5, height: 9 } }, device_width: 7, start_index: 515 }, BdfGlyph { character: 'M', bounding_box: Rectangle { top_left: Point { x: 0, y: -7 }, size: Size { width: 5, height: 9 } }, device_width: 7, start_index: 560 }, BdfGlyph { character: 'b', bounding_box: Rectangle { top_left: Point { x: 0, y: -7 }, size: Size { width: 5, height: 9 } }, device_width: 7, start_index: 605 }, BdfGlyph { character: 'i', bounding_box: Rectangle { top_left: Point { x: 0, y: -7 }, size: Size { width: 3, height: 9 } }, device_width: 5, start_index: 650 }, BdfGlyph { character: '↑', bounding_box: Rectangle { top_left: Point { x: 3, y: -8 }, size: Size { width: 5, height: 11 } }, device_width: 14, start_index: 677 }, BdfGlyph { character: '↓', bounding_box: Rectangle { top_left: Point { x: 3, y: -8 }, size: Size { width: 5, height: 11 } }, device_width: 14, start_index: 732 }, BdfGlyph { character: '中', bounding_box: Rectangle { top_left: Point { x: 0, y: -8 }, size: Size { width: 11, height: 11 } }, device_width: 14, start_index: 787 }, BdfGlyph { character: '网', bounding_box: Rectangle { top_left: Point { x: 0, y: -8 }, size: Size { width: 11, height: 11 } }, device_width: 14, start_index: 908 }, BdfGlyph { character: '连', bounding_box: Rectangle { top_left: Point { x: 0, y: -8 }, size: Size { width: 11, height: 11 } }, device_width: 14, start_index: 1029 }];

    /// maybe you want store it in special secion(e.g. .eeprom), you can use below attributes
    /// ```no_run
    /// #[no_mangle]
    /// #[link_section = ".eeprom"]
    /// ```
    static S_DATA: [u8;144] = [221, 24, 198, 49, 140, 92, 234, 170, 186, 49, 8, 136, 136, 125, 209, 136, 76, 24, 197, 194, 49, 148, 169, 124, 66, 252, 33, 232, 132, 49, 115, 163, 15, 70, 49, 139, 191, 16, 136, 68, 33, 8, 232, 198, 46, 140, 98, 231, 70, 49, 139, 195, 23, 122, 49, 143, 163, 24, 249, 209, 140, 47, 24, 205, 177, 148, 169, 138, 74, 81, 140, 119, 186, 214, 49, 140, 33, 232, 198, 49, 143, 144, 201, 36, 185, 29, 82, 16, 132, 33, 8, 66, 16, 132, 33, 8, 74, 184, 128, 128, 16, 127, 248, 67, 8, 97, 12, 33, 255, 224, 128, 16, 2, 15, 255, 0, 98, 45, 85, 145, 50, 38, 68, 213, 90, 171, 0, 96, 60, 16, 95, 192, 128, 40, 111, 228, 32, 132, 23, 242, 16, 64, 23, 252];
    
    /// glyphs is [BdfGlyph;22], data is [u8;144]. 
    /// glyphs code include: "1↓连↑3网中G0K5Mi9.8B6b742"
    /// orig bdf file is ./zpix.bdf 
    pub static  FONT_ZPIX: BdfFont = BdfFont{
        glyphs: &s_glyphs,
        data : &S_DATA,
        line_height: 12,
        replacement_character:0,
    };
}    
