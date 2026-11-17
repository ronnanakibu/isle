//! Standalone preview generator for Apple Dynamic Island all pages
use isle::theme::*;
use isle::spring::*;
use isle::mascot::*;
use isle::pages::*;
use isle::render::*;
use isle::media::MediaSnapshot;

fn save_bmp(pixels: &[u32], path: &str) {
    let mut bmp_data = Vec::with_capacity((CANVAS_WIDTH * CANVAS_HEIGHT * 4 + 54) as usize);
    let file_size = 54 + CANVAS_WIDTH * CANVAS_HEIGHT * 4;
    bmp_data.extend_from_slice(b"BM");
    bmp_data.extend_from_slice(&(file_size as u32).to_le_bytes());
    bmp_data.extend_from_slice(&[0, 0, 0, 0]);
    bmp_data.extend_from_slice(&54u32.to_le_bytes());

    bmp_data.extend_from_slice(&40u32.to_le_bytes());
    bmp_data.extend_from_slice(&(CANVAS_WIDTH as i32).to_le_bytes());
    bmp_data.extend_from_slice(&(-(CANVAS_HEIGHT as i32)).to_le_bytes());
    bmp_data.extend_from_slice(&1u16.to_le_bytes());
    bmp_data.extend_from_slice(&32u16.to_le_bytes());
    bmp_data.extend_from_slice(&0u32.to_le_bytes());
    bmp_data.extend_from_slice(&((CANVAS_WIDTH * CANVAS_HEIGHT * 4) as u32).to_le_bytes());
    bmp_data.extend_from_slice(&0u32.to_le_bytes());
    bmp_data.extend_from_slice(&0u32.to_le_bytes());
    bmp_data.extend_from_slice(&0u32.to_le_bytes());
    bmp_data.extend_from_slice(&0u32.to_le_bytes());

    for &p in pixels {
        bmp_data.extend_from_slice(&p.to_le_bytes());
    }

    std::fs::write(path, bmp_data).unwrap();
}

fn main() {
    let mut surface = Surface::new(CANVAS_WIDTH, CANVAS_HEIGHT);
    let mut w_spring = Spring::new(EXPANDED_WIDTH);
    let mut h_spring = Spring::new(EXPANDED_HEIGHT);
    w_spring.current = EXPANDED_WIDTH;
    h_spring.current = EXPANDED_HEIGHT;

    let mascot = Mascot::new();
    let media = MediaSnapshot::default();

    for (i, &page) in PageId::ALL.iter().enumerate() {
        let mut page_state = PageState::default();
        page_state.current_page = page;

        render_apple_island(
            &mut surface,
            &w_spring,
            &h_spring,
            1.0,
            &mascot,
            &page_state,
            &media,
            false,
            HitTarget::None,
        );

        let path = format!(
            r"C:\Users\Ronn\.gemini\antigravity\brain\589587cb-ddf6-46a7-93df-ea123b72604f\scratch\apple_page_{}.bmp",
            i
        );
        save_bmp(&surface.pixels, &path);
        println!("Saved {} successfully!", path);
    }
}
