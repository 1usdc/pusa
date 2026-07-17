//! Web（WASM）入口：`dx serve` 默认对此 crate 构建。
use ui::App;

fn main() {
    dioxus::LaunchBuilder::new().launch(App);
}
