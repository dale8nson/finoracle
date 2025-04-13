use crate::STOCK_INFO;
use dioxus::prelude::*;
use serde_json::{Map, Number, Value};
// use std::env;

// fn get_api_key() -> String {
//     let vars = &mut env::vars();
//     vars.find(|var| var.0 == "API_KEY")
//         .unwrap_or((String::from("API_KEY"), String::from("")))
//         .1
// }

#[component]
pub fn StockView(symbol: Signal<(String, String)>) -> Element {
    let api_key: &'static str = env!("FINNHUB_API_KEY");

    let ak = api_key.to_owned();
    let stock = use_resource(move || {
        let ak = ak.clone();
        async move { get_stock_quote(symbol().0, ak).await }
    });

    let ak = api_key.to_owned();
    let financials = use_resource(move || {
        let ak = ak.clone();
        async move { get_basic_financials(symbol().0, ak.to_string()).await }
    });

    let quote = stock
        .read_unchecked()
        .to_owned()
        .unwrap_or(Ok(Map::<String, Value>::new()))
        .unwrap_or(Map::<String, Value>::new());

    let current_price = quote
        .to_owned()
        .get("c")
        .unwrap_or(&Value::Number(Number::from_f64(0.0).unwrap()))
        .as_f64()
        .unwrap()
        .to_string();

    let opening_price = quote
        .to_owned()
        .get("o")
        .unwrap_or(&Value::Number(Number::from_f64(0.0).unwrap()))
        .as_f64()
        .unwrap()
        .to_string();
    let previous_close = quote
        .to_owned()
        .get("pc")
        .unwrap_or(&Value::Number(Number::from_f64(0.0).unwrap()))
        .as_f64()
        .unwrap()
        .to_string();
    let high = quote
        .to_owned()
        .get("h")
        .unwrap_or(&Value::Number(Number::from_f64(0.0).unwrap()))
        .as_f64()
        .unwrap()
        .to_string();
    let low = quote
        .to_owned()
        .get("l")
        .unwrap_or(&Value::Number(Number::from_f64(0.0).unwrap()))
        .as_f64()
        .unwrap()
        .to_string();
    let change = quote
        .to_owned()
        .get("d")
        .unwrap_or(&Value::Number(Number::from_f64(0.0).unwrap()))
        .as_f64()
        .unwrap_or(-9999.0)
        .to_string();
    let change_percent = quote
        .to_owned()
        .get("dp")
        .unwrap_or(&Value::Number(Number::from_f64(0.0).unwrap()))
        .as_f64()
        .unwrap_or(-9999.0)
        .to_string();

    let metrics = financials
        .read_unchecked()
        .to_owned()
        .unwrap_or(Ok(<Map<String, Value>>::new()))
        .ok()
        .unwrap()
        .get("metric")
        .unwrap_or(&Value::Object(Map::<String, Value>::new()))
        .to_owned();

    let desc = financials
        .read_unchecked()
        .to_owned()
        .unwrap_or(Ok(<Map<String, Value>>::new()))
        .ok()
        .unwrap()
        .get("description")
        .unwrap_or(&Value::Object(Map::<String, Value>::new()))
        .to_owned()
        .as_str()
        .unwrap_or(format!("{quote:?}").as_str())
        .to_owned();

    rsx! {
             div {class:"w-[100%] h-[100%] flex flex-col p-[0.75rem] m-auto w-[100%] overflow-y-scroll relative",
                 div {position:"sticky",
                     h2 {class:"text-[#ffffff] text-center text-[1.5rem] font-bold",{format!("{}", symbol().1)}}
                     h3 {class:"text-[#ffffff] text-[1rem] text-center", {format!("{}", symbol().0)}}
                 }

             div {class:"grid grid-cols-[8fr_1fr] gap-x-[3rem] h-[100%] w-[100%]", visibility: if STOCK_INFO().get("symbol") == None {"hidden"} else {"visible"},
                 h3 {"Current Price"}
                 p {"${current_price}"}
                 h3 {"Previous Close"}
                 p {"${previous_close}"}
                 h3 {"Opening Price"}
                 p {"${opening_price}"}
                 h3 {"Change"}
                 p {"${change}"}
                 h3 {"Change Percent"}
                 p {"{change_percent}%"}
                 h3 {"High"}
                 p {"${high}"}
                 h3 {"Low"}
                 p {"${low}"}
                 for (k, v) in metrics.as_object().unwrap().into_iter() {
                     div {class:"flex flex-col justify-center items-start", h3 {{k.chars().fold(String::from(""), |a, e|
                         {
                             let len = &a.len();
                             if *len > 0  {
                                 let prev_c = a.chars().nth(len - 1).unwrap();
                                 if ((prev_c.is_alphabetic() && prev_c.is_lowercase()) || e.is_alphabetic() &&
                                     prev_c.is_digit(10))
                                 && e == e.to_ascii_uppercase()
                                 {a + " " + &e.to_string()}
                                 else { a + &e.to_string()}
                             }
                             else {
                                 a + e.to_ascii_uppercase().to_string().as_str()
                             }
                         }
                     )}}}
                     div {class:"flex flex-col justify-center items-start",
                         p {{  match v.to_owned() {
                             Value::Number(n) => n.to_string(),
                             Value::String(s) => s,
                             _ => String::new()
                         }}}
                    }}

             // p{"{financials:?}"}
             }
        }
    }
}

#[server]
async fn get_stock_quote(
    symbol: String,
    api_key: String,
) -> Result<Map<String, Value>, ServerFnError> {
    let body = reqwest::get(&format!(
        "https://finnhub.io/api/v1/quote?symbol={}&token={api_key}",
        symbol
    ))
    .await?
    .text()
    .await?;
    let quote: serde_json::Value = serde_json::from_str(&body)?;
    Ok(quote.as_object().unwrap().clone())
}

#[server]
async fn get_basic_financials(
    symbol: String,
    api_key: String,
) -> Result<Map<String, Value>, ServerFnError> {
    let body = reqwest::get(&format!(
        "https://finnhub.io/api/v1/stock/metric?symbol={}&metric=all&token={api_key}",
        symbol
    ))
    .await?
    .text()
    .await?;
    let financials: serde_json::Value = serde_json::from_str(&body)?;
    Ok(financials.as_object().unwrap().clone())
}
