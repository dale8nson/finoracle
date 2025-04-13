use crate::COUNTRY;
use dioxus::prelude::*;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
// use std::env;
// use std::env::Vars;

pub static STOCK_INFO: GlobalSignal<Map<String, Value>> =
    Global::new(|| Map::<String, Value>::new());

#[component]
pub fn StockList(symbol: Signal<(String, String)>) -> Element {
    let api_key: &'static str = env!("FINNHUB_API_KEY");

    let mut search_term = use_signal(|| String::from(""));

    let country = use_signal(|| String::from("US"));

    let onvaluechange = move |e: Event<FormData>| {
        *search_term.write() = e.value().to_uppercase();
    };

    // let mut vars = env::vars();
    // let api_key = vars
    //     .find(|var| var.0.as_str() == "API_KEY")
    //     .unwrap_or((String::from("API_KEY"), String::from("")));

    // let ak = api_key.1.to_owned();
    // let ak = ak.to_owned();
    // use_effect(move || {
    //     *API_KEY.write() = ak.to_owned();
    // });

    // let ak = &api_key.1.to_owned();
    // let ak = ak.to_owned();
    let symbols = use_resource(move || {
        // let country = COUNTRY();
        let ak = api_key.clone();
        async move { fetch_symbols(country(), ak.to_string()).await }
    });

    let value = Value::String("".to_string());
    let mut stock_list: BTreeMap<String, String> = {
        let s = &*symbols.read_unchecked();
        if let Some(s) = s {
            // api_key = s.1.clone();
            if let Ok(s) = s {
                s.iter()
                    .map(|s| {
                        let symbol = &s
                            .get("symbol")
                            .unwrap_or(&value)
                            .as_str()
                            .unwrap_or("")
                            .to_string();
                        let description = &s
                            .get("description")
                            .unwrap_or(&value)
                            .as_str()
                            .unwrap_or("")
                            .to_string();
                        (symbol.to_owned(), description.to_owned())
                    })
                    // .collect::<Vec<(String, String)>>()
                    .into_iter()
                    .collect::<BTreeMap<String, String>>()
            } else {
                BTreeMap::<String, String>::new()
            }
        } else {
            BTreeMap::<String, String>::new()
        }
    };

    let find_match = move |sym: String| match &*symbols.read_unchecked() {
        Some(Ok(s)) => s
            .into_iter()
            .find(|map| map["symbol"] == sym)
            .unwrap_or(&Map::<String, Value>::new())
            .to_owned(),
        Some(Err(_)) => Map::<String, Value>::new(),
        _ => Map::<String, Value>::new(),
    };

    // let stock_list = stock_list.unwrap_or(Vec::<(String, String)>::new());
    // stock_list.sort();

    let stock_list = stock_list.into_iter().filter(|(symbol, desc)| {
        symbol.as_str().contains(search_term().as_str())
            || desc.as_str().contains(search_term().as_str())
    });

    rsx! {
        div { class:"flex flex-col justify-start items-center w-[100%] h-[99.5%]",
            div {border_bottom:"solid #fff", border_right: "none", border_top: "none", class:"flex flex-row justify-between items-center w-[100%] h-[4rem] overflow-y-clipped",
                input {onchange:onvaluechange, border_bottom:"solid #fff",  class:"mt-[0.6rem] px-[1rem] h-[95%] py-[0.25rem] w-[100%] text-[1.5rem] border-none  fixed z-[10] top-0 left-0 bg-[#000000] text-[#ffffff] relative", placeholder:"Search for a symbol..."}
                div { class:"grid grid-cols-4 px-[1rem] h-[100%] gap-x-[0.25rem] items-center w-[60%]",
                    div {class:"flex flex-col justify-center items-center",
                        label {for:"US", "US"}
                        input{id:"US", r#type:"radio", value:"US", checked: COUNTRY() == "US".to_string(), onchange:move |_| *COUNTRY.write() = "US".into()}}
                    div {class:"flex flex-col justify-center items-center",
                        label {for:"AU", "AU"}
                        input{id:"AU", disabled:true, r#type:"radio", value:"AU", checked: COUNTRY() == "AU".to_string(), onchange:move |_| *COUNTRY.write() = "AU".into()}}
                    div {class:"flex flex-col justify-center items-center",
                        label {for:"UK", "UK"}
                        input{id:"UK", disabled:true, r#type:"radio", value:"UK" , checked: COUNTRY() == "GB".to_string(), onchange:move |_| *COUNTRY.write() =  "GB".into()}}
                    div {class:"flex flex-col justify-center items-center", label {for:"JP", "JP"}
                        input{id:"JP", disabled:true, r#type:"radio", value:"JP", checked: COUNTRY() == "JP".to_string(), onchange:move |_| *COUNTRY.write() = "JP".into() }
                    }
                }
            }
            div {class:"border-[#ffffff] w-[100%] h-[90%] flex flex-col p-[0.125rem] overflow-x-hidden",
                ul {class:"w-[100%] max-h-[90%] list-none list-outside ml-[0px] pl-[0.5rem]",
            {stock_list.map(|(sym, desc)| {
                rsx! {
                        li { class: "text-[#ffffff] bg-[#000000] my-[0px] w-full",
                            button {color: if sym.to_owned() == symbol().0 {"#0000ee"} else {"#ffffff"} , onclick:move |_| { symbol.set((sym.to_owned(), desc.to_owned())); *STOCK_INFO.write() = find_match(sym.to_owned())},class:"border-none grid-cols-4 hover:cursor-pointer hover:text-[#0000ee] bg-[#000000] text-left text-[#ffffff] w-[100%] font-bold text-[1.125rem] my-[0px] py-0 mx-[5px]", display:"grid",
                                div {class:"text-left flex flex-row justify-start items-center", {sym.to_owned()}}
                        div {class:"flex col-span-3 flex-row justify-start items-center text-left", {desc.to_owned()}}
                            }
                        }
                }

            })
            }
                }
            }
        }
    }
}

#[server]
async fn fetch_symbols(
    country: String,
    api_key: String,
) -> Result<Vec<Map<String, Value>>, ServerFnError> {
    let url = format!("https://finnhub.io/api/v1/stock/symbol?exchange={country}&token={api_key}");
    let body = reqwest::get(&url).await?.text().await?;
    let symbols: serde_json::Value = serde_json::from_str(&body)?;
    let symbols = Vec::<Map<String, Value>>::from(
        symbols
            .as_array()
            .unwrap_or(&Vec::<Value>::new())
            .iter()
            .map(|s| {
                s.as_object()
                    .unwrap_or(&Map::<String, Value>::new())
                    .to_owned()
            })
            .collect::<Vec<Map<String, Value>>>(),
    );

    Ok(symbols)
}
