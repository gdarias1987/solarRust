//! Windows only: compiles `assets/app.rc` so the executable carries the Sun as its icon
//! resource. Explorer, Start, pinning and the taskbar's group icon read that resource; the
//! running window's own icon still comes from `rune::set_app_icon`. Other targets build without
//! the resource compiler.

fn main() {
    println!("cargo:rerun-if-changed=assets/app.rc");
    println!("cargo:rerun-if-changed=assets/app.ico");
    #[cfg(windows)]
    let _ = embed_resource::compile("assets/app.rc", embed_resource::NONE);
}
