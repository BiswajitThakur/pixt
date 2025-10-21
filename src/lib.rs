pub mod img;
pub mod render;
pub mod style;

use wasm_bindgen::prelude::*;
use web_sys::{
    Document, Event, FileReader, HtmlElement, HtmlInputElement, HtmlSelectElement, Window,
    js_sys::{self},
};

use crate::{
    img::{IntoPixtData, OutputType, PixtImg},
    render::render,
    style::ImgStyle,
};
#[wasm_bindgen]
pub fn start() {
    MyPage::new().unwrap().handle_input();
}

#[derive(Clone)]
struct MyPage {
    window: Window,
    document: Document,
}

impl MyPage {
    fn new() -> Option<Self> {
        let window = web_sys::window()?;
        let document = window.document()?;
        Some(Self { window, document })
    }
    fn handle_input(&self) {
        self.init();
        self.handle_convert_btn();
        self.handle_image_input();
    }
    fn alert(&self, msg: &str) {
        self.window.alert_with_message(msg).unwrap();
    }
    fn init(&self) {
        let global = web_sys::js_sys::global();
        js_sys::Reflect::set(
            &global,
            &JsValue::from_str("img_width"),
            &JsValue::from_f64(0.0),
        )
        .unwrap();
        js_sys::Reflect::set(
            &global,
            &JsValue::from_str("img_height"),
            &JsValue::from_f64(0.0),
        )
        .unwrap();
        let array_buffer = js_sys::Uint8Array::new(&JsValue::null());
        js_sys::Reflect::set(&global, &JsValue::from_str("image_data"), &array_buffer).unwrap();
    }

    fn handle_convert_btn(&self) {
        let document = self.document.clone();
        let output = document
            .get_element_by_id("output")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        let convert_btn = self
            .document
            .get_element_by_id("convertBtn")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        let select = self
            .document
            .get_element_by_id("out_img_style")
            .unwrap()
            .dyn_into::<HtmlSelectElement>()
            .unwrap();
        let f = Closure::<dyn FnMut()>::new(move || {
            let global = web_sys::js_sys::global();
            let img_data = match js_sys::Reflect::get(&global, &JsValue::from_str("image_data")) {
                Ok(v) => v,
                Err(_) => return,
            };
            output.set_inner_text("Converting........");
            let array_buffer = js_sys::Uint8Array::new(&img_data);
            let img = image::load_from_memory(&array_buffer.to_vec())
                .unwrap()
                .resize(
                    get_img_width(&document),
                    get_img_height(&document),
                    image::imageops::FilterType::Nearest,
                );
            let mut out = Vec::new();
            let pix_img: PixtImg = match select.value().as_str() {
                "ascii" => PixtImg::new(ImgStyle::Ascii, OutputType::text()),
                "block" => PixtImg::new(ImgStyle::Block, OutputType::text()),
                "pixel" => PixtImg::new(ImgStyle::Pixel, OutputType::text()),
                "braills" => PixtImg::new(ImgStyle::Braills, OutputType::text()),
                "dots" => PixtImg::new(ImgStyle::Dots, OutputType::text()),
                "custom" => {
                    let e = document
                        .get_element_by_id("custom_ascii_input")
                        .unwrap()
                        .dyn_into::<HtmlInputElement>()
                        .unwrap();
                    let v = e.value();
                    if v.is_empty() {
                        return;
                    }
                    PixtImg::new(v.chars().collect::<Vec<char>>(), OutputType::text())
                }
                _ => unreachable!(),
            };
            render(&pix_img, &img, &mut out).unwrap();
            let out = unsafe { str::from_utf8_unchecked(&out) };
            output.set_inner_text(out);
        });
        convert_btn.set_onclick(Some(f.as_ref().unchecked_ref()));
        f.forget();
    }
    fn handle_image_input(&self) {
        let image_input: HtmlInputElement = self
            .document
            .get_element_by_id("imageInput")
            .unwrap()
            .dyn_into()
            .unwrap();
        let img_width = self
            .document
            .get_element_by_id("widthInput")
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();

        let img_height = self
            .document
            .get_element_by_id("heightInput")
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();
        let image_input_clone = image_input.clone();
        let on_change = Closure::<dyn FnMut(_)>::new(move |_event: Event| {
            let img_width = img_width.clone();
            let img_height = img_height.clone();

            if let Some(files) = image_input_clone.files() {
                if let Some(file) = files.get(0) {
                    let reader = FileReader::new().unwrap();
                    let reader_clone = reader.clone();

                    let onload = Closure::<dyn FnMut(_)>::new(move |_: Event| {
                        let global = js_sys::global();
                        js_sys::Reflect::set(
                            &global,
                            &JsValue::from_str("image_data"),
                            &JsValue::null(),
                        )
                        .unwrap();
                        let result = reader_clone.result().unwrap();
                        let array_buffer = js_sys::Uint8Array::new(&result);
                        let img = image::load_from_memory(&array_buffer.to_vec()).unwrap();
                        js_sys::Reflect::set(
                            &global,
                            &JsValue::from_str("image_data"),
                            &array_buffer,
                        )
                        .unwrap();
                        js_sys::Reflect::set(
                            &global,
                            &JsValue::from_str("img_width"),
                            &JsValue::from_f64(img.width() as f64),
                        )
                        .unwrap();
                        js_sys::Reflect::set(
                            &global,
                            &JsValue::from_str("img_height"),
                            &JsValue::from_f64(img.height() as f64),
                        )
                        .unwrap();
                        img_width.set_value(img.width().to_string().as_str());
                        img_height.set_value(img.height().to_string().as_str());
                    });
                    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                    reader.read_as_array_buffer(&file).unwrap();
                    onload.forget();
                }
            }
        });
        image_input
            .add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref())
            .unwrap();
        on_change.forget();
    }
}

fn get_img_width(document: &Document) -> u32 {
    let img_width = document
        .get_element_by_id("widthInput")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    img_width.value().parse().unwrap_or_default()
}

fn get_img_height(document: &Document) -> u32 {
    let img_width = document
        .get_element_by_id("heightInput")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    img_width.value().parse().unwrap_or_default()
}
