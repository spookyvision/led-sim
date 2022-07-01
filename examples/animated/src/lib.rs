#![feature(generic_const_exprs)]
use std::{cell::RefCell, rc::Rc};

use eg_bdf::{include_bdf, text::BdfTextStyle, BdfFont};
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{Rgb565, Rgb888},
    prelude::{Point, Primitive, WebColors, *},
    primitives::{Circle, PrimitiveStyle, Rectangle},
    text::Text,
    Drawable,
};
use embedded_graphics_web_simulator::{
    display::WebSimulatorDisplay, output_settings::OutputSettingsBuilder,
};
use graphics::BufDisplay;
use wasm_bindgen::{prelude::*, JsCast};

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

const NUM_ITER: i32 = 6000;

const AW_MINTS: BdfFont = include_bdf!("/Users/ace/play/clone/artwiz-fonts-wl/mints-mild-bold.bdf");

// This function is automatically invoked after the wasm module is instantiated.
#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let aw_style = BdfTextStyle::new(&AW_MINTS, Rgb888::MAGENTA);

    let document = document();
    let output_settings = OutputSettingsBuilder::new()
        .scale(24)
        .pixel_spacing(3)
        .build();
    let size = (2 * NUM_ITER) as u32;

    const W: usize = 7;
    const H: usize = 7;

    const AA: bool = true;

    let mut img_display = WebSimulatorDisplay::new(
        (W as u32, H as u32),
        &output_settings,
        document.get_element_by_id("graphics").as_ref(),
    );

    let mut buffy = BufDisplay::<W, H>::new();
    // buffy.set_alpha_div(2);

    // Here we want to call `requestAnimationFrame` in a loop, but only a fixed
    // number of times. After it's done we want all our resources cleaned up. To
    // achieve this we're using an `Rc`. The `Rc` will eventually store the
    // closure we want to execute on each frame, but to start out it contains
    // `None`.
    //
    // After the `Rc` is made we'll actually create the closure, and the closure
    // will reference one of the `Rc` instances. The other `Rc` reference is
    // used to store the closure, request the first frame, and then is dropped
    // by this function.
    //
    // Inside the closure we've got a persistent `Rc` reference, which we use
    // for all future iterations of the loop
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut i = 0;
    let mut i2 = 1;

    let mut fancy_backing = [rgb::RGB8::default(); { W * H }];

    let mut rand1 = randomize::PCG32::seed(13, 19);
    let mut rand2 = randomize::PCG32::seed(253, 17);

    let mut row = effects::Row::new(
        (64 - rand1.next_u32() % 32) as u8,
        0,
        1,
        H.try_into().unwrap(),
        (rand2.next_u32() % 5) as u8,
    );

    let mut rows = vec![];
    // fn new(speed: u8, x: u8, y: u8, height: u8) -> Self {
    for i in 0..4 {
        let row = effects::Row::new(
            (60 - rand1.next_u32() % 28) as u8,
            (i % W) as u8,
            (rand2.next_u32() % (H as u32 / 2)) as u8,
            H as u8,
            (10 + rand2.next_u32() % 30) as u8,
        );
        rows.push(row);
    }

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let terminate = false;

        if terminate {
            // Drop our handle to this closure so that it will get cleaned
            // up once we return.
            let _ = f.borrow_mut().take();
            return;
        }

        img_display.clear(Rgb888::BLACK).expect("could not clear()");

        // let area = Rectangle::new(Point::new(0, 0), Size::new(W, H));

        // let pixels = area.points().map(|point| {
        //     let xy_extrawurst = Vector2::new(point.x as f32, point.y as f32);
        //     Pixel(point, qc.pixel(xy_extrawurst, time))
        // });

        // img_display.draw_iter(pixels).expect("could not draw..?");

        let moddor = 35;
        let scroll = i / 2;
        // let x = 10 - (scroll % moddor);
        // let x2 = 10 - ((i2 / 2) % moddor);
        // let y = 6;

        buffy.clear(Rgb888::BLACK).expect("what?");

        effects::expanding_circle_2(
            ((i % 32) as f32 * 0.3) as u8,
            (i / 3).max(1).min(1) as u8,
            3,
            &mut fancy_backing,
            false,
        );
        // buffy.blit(&mut fancy_backing);

        // let _position = Text::new("CHR ", Point::new(x, y), aw_style)
        //     .draw(&mut buffy)
        //     .unwrap();

        // effects::chaser(i as u16, &mut backing);

        // let it = backing
        //     .iter()
        //     .enumerate()
        //     .filter_map(|(idx, val)| match val {
        //         &Rgb888::BLACK => None,
        //         _ => Some(Pixel(
        //             Point::new((idx % W) as i32, (idx / W) as i32),
        //             Rgb888::new(val.r(), val.g(), val.b()),
        //         )),
        //     });

        // img_display.draw_iter(it).expect("no drawing the thing?");

        img_display.draw_iter(buffy.iter_pixels());

        for row in &mut rows {
            let dies = row.tick();

            if dies {
                *row = effects::Row::new(
                    (60 - rand1.next_u32() % 28) as u8,
                    (i % W) as u8,
                    (rand2.next_u32() % (H as u32 / 2)) as u8,
                    H as u8,
                    (10 + rand2.next_u32() % 30) as u8,
                );
            } else {
                img_display.draw_iter(row.iter());
            }
        }

        img_display.flush().expect("could not flush buffer");

        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
        i += 1;
        if i % 2 == 0 {
            i2 += 1;
        }
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}
