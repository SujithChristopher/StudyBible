use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component] 
fn App() -> Element {
    rsx! {
        div { 
            class: "min-h-screen flex bg-gray-50 p-8",
            h1 {
                class: "text-2xl font-bold text-gray-900",
                "StudyBible - Minimal Test"
            }
            p {
                class: "text-gray-600 mt-4", 
                "This is a minimal test to debug the crash issue."
            }
        }
    }
}