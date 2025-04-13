use dioxus::prelude::*;

#[component]
pub fn ListView(class: &'static str, children: Element) -> Element {
    rsx! {
        div {class:" w-[100%] h-[100%] overflow-y-scroll overflow-x-hidden {class}",
        ul {
    class: "list-none list-outside relative gap-y-[0px] m-[0px] top-[0.5rem] px-[0.5rem] pt-[0px] w-[100%] h-[90%] z-[1]",
    {children}}
    }
    }
}
