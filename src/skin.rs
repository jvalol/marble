//! What the marble is painted, so you can see it turning. See
//! `specs/0006-it-rolls.md`.
//!
//! Spec 0030 gives the ball spin, and spin on a plain coloured sphere looks
//! exactly like no spin at all. A pattern is the whole of what makes rolling
//! visible, and it is built here rather than shipped as a file because two
//! colours in a grid is cheaper to write than to load.

use blitzkit::texture::TextureData;

/// How many squares around the ball, and how big the image is.
pub const SQUARES: u32 = 8;
pub const SIZE: u32 = 128;

/// The two colours, light and dark, as straight RGBA.
pub const LIGHT: [u8; 4] = [242, 238, 230, 255];
pub const DARK: [u8; 4] = [48, 52, 64, 255];

/// A checker, which reads as turning from any angle.
///
/// Stripes would read as turning about one axis and as nothing about the
/// others, and a ball on a course turns about all of them.
pub fn checker() -> TextureData {
    TextureData::from_pixels(SIZE, SIZE, pattern())
}

/// The squares themselves, as straight RGBA.
///
/// Separate from the texture because a `TextureData` keeps its pixels to
/// itself, and a pattern nothing can read is a pattern nothing can check.
pub fn pattern() -> Vec<u8> {
    let step = SIZE / SQUARES;
    let mut pixels = Vec::with_capacity((SIZE * SIZE * 4) as usize);

    for y in 0..SIZE {
        for x in 0..SIZE {
            let dark = ((x / step) + (y / step)).is_multiple_of(2);
            pixels.extend_from_slice(if dark { &DARK } else { &LIGHT });
        }
    }

    pixels
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(pixels: &[u8], x: u32, y: u32) -> [u8; 4] {
        let at = ((y * SIZE + x) * 4) as usize;

        [pixels[at], pixels[at + 1], pixels[at + 2], pixels[at + 3]]
    }

    #[test]
    fn it_is_the_size_it_says() {
        let skin = checker();

        assert_eq!(skin.width(), SIZE);
        assert_eq!(skin.height(), SIZE);
        assert_eq!(pattern().len(), (SIZE * SIZE * 4) as usize);
    }

    #[test]
    fn next_door_squares_differ() {
        // a pattern that does not change along the ball says nothing about
        // whether the ball is turning
        let step = SIZE / SQUARES;
        let pixels = pattern();
        let pixels = pixels.as_slice();

        let here = pixel(pixels, 0, 0);
        assert_ne!(here, pixel(pixels, step, 0), "across");
        assert_ne!(here, pixel(pixels, 0, step), "down");
        assert_eq!(here, pixel(pixels, step, step), "diagonally");
    }

    #[test]
    fn it_is_half_one_and_half_the_other() {
        let dark = pattern()
            .chunks(4)
            .filter(|pixel| pixel[0] == DARK[0])
            .count();

        assert_eq!(dark * 2, (SIZE * SIZE) as usize);
    }

    #[test]
    fn it_is_opaque() {
        assert!(pattern().chunks(4).all(|pixel| pixel[3] == 255));
    }
}
