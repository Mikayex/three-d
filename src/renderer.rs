
#[cfg(target_os = "emscripten")]
use emscripten::{emscripten};

#[cfg(target_os = "emscripten")]
pub fn set_main_loop<F>(main_loop: F) where F: FnMut()
{
    emscripten::set_main_loop_callback(main_loop);
}

#[cfg(not(target_os = "emscripten"))]
pub fn set_main_loop<F>(mut main_loop: F) where F: FnMut()
{
    loop { main_loop(); }
}

#[cfg(not(target_os = "emscripten"))]
pub fn load<F>(name: &str, mut on_load: F) where F: FnOnce(String)
{
    use loader;
    let data = loader::load_string(name).unwrap();
    on_load(data);
}

