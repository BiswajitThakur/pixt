pub mod img;
pub mod render;
pub mod style;

use std::io;

use image::{DynamicImage, ImageReader};
use wasm_bindgen::prelude::*;
use web_sys::{
    Document, Event, FileList, FileReader, HtmlElement, HtmlInputElement,
    js_sys::{self, Function},
};
#[wasm_bindgen]
pub fn start() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let convert_btn = document
        .get_element_by_id("convertBtn")
        .unwrap()
        .dyn_into::<HtmlElement>()
        .unwrap();

    let image_input: HtmlInputElement = document
        .get_element_by_id("imageInput")
        .unwrap()
        .dyn_into()
        .unwrap();

    let img_width = document
        .get_element_by_id("widthInput")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap();

    let img_height = document
        .get_element_by_id("heightInput")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap();

    let keep_ratio = document
        .get_element_by_id("keepRatio")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap();

    let cloned_window = window.clone();
    let image_input_clone = image_input.clone();
    let img_width_clone = img_width.clone();
    let img_height_clone = img_height.clone();

    let on_change = Closure::<dyn FnMut(_)>::new(move |_event: Event| {
        let img_width = img_width_clone.clone();
        let img_height = img_height_clone.clone();

        if let Some(files) = image_input_clone.files() {
            if let Some(file) = files.get(0) {
                let reader = FileReader::new().unwrap();
                let reader_clone = reader.clone();

                let onload = Closure::<dyn FnMut(_)>::new(move |_: Event| {
                    let result = reader_clone.result().unwrap();
                    let array_buffer = js_sys::Uint8Array::new(&result);
                    let img = image::load_from_memory(&array_buffer.to_vec()).unwrap();
                    let global = js_sys::global();
                    js_sys::Reflect::set(&global, &JsValue::from_str("image_data"), &array_buffer)
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
    let f = Closure::<dyn FnMut()>::new(move || {
        cloned_window
            .alert_with_message(
                format!(
                    "{} x {}",
                    get_img_width(&document),
                    get_img_height(&document)
                )
                .as_str(),
            )
            .unwrap();
    });
    convert_btn.set_onclick(Some(f.as_ref().unchecked_ref()));
    f.forget();
}

fn get_img_width_element(document: &Document) -> HtmlInputElement {
    document
        .get_element_by_id("widthInput")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap()
}

fn get_img_height_element(document: &Document) -> HtmlInputElement {
    document
        .get_element_by_id("heightInput")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap()
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

fn get_img_data(document: &Document) -> Option<Vec<u8>> {
    todo!()
}
