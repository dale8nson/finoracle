mod components;
use crate::STOCK_INFO;
use components::*;
use dioxus::prelude::*;

pub static COUNTRY: GlobalSignal<String> = Global::new(|| String::from("US"));
pub static LOG: GlobalSignal<String> = Global::new(|| String::from(""));

// use dioxus_desktop::{tao::window::Fullscreen, Config, WindowBuilder};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    // #[layout(Navbar)]
    #[route("/")]
    Home {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    // #[cfg(feature = "desktop")]
    // let _ = dioxus::LaunchBuilder::desktop()
    //     .with_cfg(
    //         Config::new().with_window(
    //             WindowBuilder::new()
    //                 .with_fullscreen(Some(Fullscreen::Borderless(None)))
    //                 .with_title("FinOracle"),
    //         ),
    //     )
    //     .launch(App);
    //
    // let env_var = std::fs::File::open("env.json");
    // match env_var {
    //     Ok(mut env_var) => {
    //         *LOG.write() = String::from("env.json file successfully opened");
    //         let mut json = String::new();
    //         let res = env_var.read_to_string(&mut json);
    //         if let Ok(_) = res {
    //             if let Ok(res) = serde_json::from_str(&json) {
    //                 *ENV.write() = res;
    //             }
    //         }
    //     }
    //     Err(e) => *LOG.write() = e.to_string(),
    // }
    // #[cfg(feature = "web")]
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS } document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        div {class:"w-[100%] h-[95vh] flex justify-center items-center p-8 text-[#ffffff] font-[sans-serif] bg-[#000000]",
            Router::<Route> {}
        }
    }
}

/// Home page
#[component]
fn Home() -> Element {
    let mut symbol = use_signal(|| (String::from(""), String::from("")));
    let size = use_signal(|| (0, 0));

    rsx! {
        main {id:"main", class:"w-[100%] h-[95vh] grid grid-cols-4 gap-x-[2rem]",
            div { class:"flex flex-col w-[100%] h-[95vh] items-center justify-start relative gap-y-[2rem]",
                div {height:"50%", class:"h-[50vh] w-[100%] flex flex-col items-start justify-start border-solid border-2 rounded-[0.75rem]",
                    // button {class:"bg-[#000000] text-[#ffffff] hover:text-[#eeeeee] hover:translate-x-[-0.2rem] duration-[0.25s] border-none text-[2rem] m-[0px]" ,"⬅︎"}
                    StockList{symbol}
                }
                div {height:"50vh", class:"border-[#ffffff] w-[100%] h-[50vh] flex flex-col justify-start border-solid border-2 rounded-[0.75rem]  overflow-x-hidden",
                    StockView{symbol}
                }
            }
            div { class:"w-[100%] h-[95vh] col-span-3  gap-y-[2rem] flex flex-col justify-center items-center",
                div {class:"px-[1rem] gap-y-[2rem] flex flex-col justify-start items-start h-[95vh] w-[100%]",
                    div {class:"w-[100%] h-[50vh]　m-[0px] flex flex-col border-[#ffffff] border-solid border-2 rounded-[0.75rem] overflow-y-hidden",
                        ChartView {symbol}
                    }
                    div {class:"border-[#ffffff] w-[100%] h-[50vh] flex flex-col border-solid border-2 rounded-[0.75rem] overflow-y-hidden",
                    }
                }
                }
        }
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div {
            class: "w-full h-auto",
            id: "navbar",
            Link {
                to: Route::Home {},
                "Home"
            }

        }
        Outlet::<Route> {}
    }
}
