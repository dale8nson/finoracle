use chrono::{Datelike, NaiveDate};
use dioxus::prelude::*;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Value};
use std::cmp::Ordering;
use std::fmt;
use std::marker::PhantomData;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[derive(Deserialize, Debug, PartialEq, PartialOrd, Clone)]
struct DataPoint {
    #[serde(deserialize_with = "str_to_date")]
    period: NaiveDate,
    v: f64,
}

impl Eq for DataPoint {}

impl Ord for DataPoint {
    fn cmp(&self, other: &DataPoint) -> Ordering {
        if self.period < other.period {
            Ordering::Less
        } else if self.period > other.period {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

fn str_to_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    struct Struct(PhantomData<fn() -> NaiveDate>);

    impl<'de> Visitor<'de> for Struct {
        type Value = NaiveDate;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("NaiveDate")
        }

        fn visit_str<E>(self, value: &str) -> Result<NaiveDate, E>
        where
            E: de::Error,
        {
            Ok(NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap_or(NaiveDate::default()))
        }
    }

    deserializer.deserialize_str(Struct(PhantomData))
}

impl From<DataPoint> for String {
    fn from(value: DataPoint) -> Self {
        let year = value.period.year();
        let month = value.period.month();
        let day = value.period.day();

        String::from(format!("{}-{}-{}", year, month, day))
    }
}

#[component]
pub fn ChartView(symbol: Signal<String>) -> Element {
    let api_key: &'static str = env!("FINNHUB_API_KEY");

    let mut selected_tab = use_signal(|| String::from("annual"));
    let mut series_btn = use_signal(|| false);
    // let mut selected_annual_series = use_signal(|| HashSet::<String>::new());
    // let mut selected_quarterly_series = use_signal(|| HashSet::<String>::new());
    let mut selected_annual_series = use_signal(|| String::from(""));
    let mut selected_quarterly_series = use_signal(|| String::from(""));
    let dummy_string = String::from("");
    let dummy_val = Value::Null;
    let mut selected_financial: Signal<Option<(String, Value)>> = use_signal(|| None);
    let mut canvas: Signal<Option<Event<MountedData>>> = use_signal(|| None);

    let ak = api_key.to_owned();
    let financials = use_resource(move || {
        let ak = ak.clone();
        async move { get_basic_financials(symbol(), ak).await }
    });

    let mut series = use_signal(|| Map::<String, Value>::new());

    let mut xmax = use_signal(|| 0i32);
    let mut xmin = use_signal(|| 0i32);
    let mut ymin = use_signal(|| 0f64);
    let mut ymax = use_signal(|| 0f64);

    use_effect(move || {
        series.set(
            financials
                .read_unchecked()
                .to_owned()
                .unwrap_or(Ok(Map::<String, Value>::new()))
                .ok()
                .unwrap()
                .get("series")
                .unwrap_or(&Value::Object(Map::<String, Value>::new()))
                .as_object()
                .unwrap_or(&Map::<String, Value>::new())
                .to_owned(),
        );
    });

    use_effect(move || {
        let ts = series()
            .into_iter()
            .find(|(k, _)| k == &String::from(selected_tab()))
            .unwrap_or((
                String::from(""),
                Value::String(String::from("Computer says no")),
            ));
        let default_map = &Map::<String, Value>::default();
        let default_tuple = (String::from(""), Value::String(String::from("")));

        let ts =
            ts.1.as_object()
                .unwrap_or(default_map)
                .to_owned()
                .into_iter()
                .find(|(k, _)| match selected_tab().as_str() {
                    "annual" => **k == selected_annual_series(),
                    "quarterly" => **k == selected_quarterly_series(),
                    _ => false,
                })
                .unwrap_or(default_tuple);

        let mut ts: Vec<DataPoint> =
            serde_json::from_value(ts.1.to_owned()).unwrap_or(Vec::<DataPoint>::new());

        ts.sort_by(|a, b| {
            if a.period < b.period {
                Ordering::Less
            } else if a.period > b.period {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });

        let mut dates = Vec::<NaiveDate>::new();
        let mut values = Vec::<f64>::new();

        for dp in &ts {
            dates.push(dp.period);
            values.push(dp.v);
        }

        let document = web_sys::window().unwrap().document().unwrap();
        let container = document.get_element_by_id("chart-container").unwrap();
        let rect = container.get_bounding_client_rect();
        let width = rect.width();
        let height = rect.height();
        let el = document.get_element_by_id("chart").unwrap();
        let canvas: HtmlCanvasElement = el.dyn_into::<HtmlCanvasElement>().map_err(|_| ()).unwrap();
        canvas.set_width(width as u32);
        canvas.set_height((height * 0.98f64) as u32);

        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        ctx.set_stroke_style_str("rgb(238, 0, 0)");
        ctx.set_fill_style_str("rgb(238,0,0)");
        ctx.set_line_width(2.0);

        ctx.move_to(
            0.1f64 * canvas.width() as f64,
            0.01f64 * canvas.height() as f64,
        );
        ctx.line_to(
            0.1f64 * canvas.width() as f64,
            0.9f64 * canvas.height() as f64,
        );
        ctx.line_to(
            0.95f64 * canvas.width() as f64,
            0.9f64 * canvas.height() as f64,
        );

        ctx.stroke();

        let default_max_date = NaiveDate::from_ymd_opt(2025, 7, 1).unwrap();
        let default_min_date = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();

        let x_max = dates.iter().max().unwrap_or(&default_max_date);
        let x_min = dates.iter().min().unwrap_or(&default_min_date);
        let y_max = values
            .iter()
            .max_by(|x, y| {
                if **x < **y {
                    Ordering::Less
                } else if **x > **y {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            })
            .unwrap_or(&5000.0);

        let y_min = values
            .iter()
            .min_by(|x, y| {
                if **x < **y {
                    Ordering::Less
                } else if **x > **y {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            })
            .unwrap_or(&0.0);

        // ctx.move_to(0.25 * canvas.width() as f64, 0.05 * canvas.height() as f64);
        ctx.set_font("20px sans-serif");
        let step = (x_max.year() as usize - x_min.year() as usize) / 5;

        if step > 0 {
            for year in (x_min.year()..=x_max.year())
                .step_by((x_max.year() as usize - x_min.year() as usize) / 5)
            {
                let x = ((year - x_min.year()) as f64 / (x_max.year() - x_min.year()) as f64)
                * (0.85 * canvas.width() as f64)
                + 0.075 * canvas.width() as f64
                // - 0.025 * canvas.width() as f64
                ;

                let _ = ctx.fill_text(
                    format!("{year}").as_str(),
                    x as f64,
                    0.95 * canvas.height() as f64,
                );
            }
        }

        let y_start = *y_min;
        // y_start.set(*y_min);
        let y_end = *y_max;
        // y_end.set(*y_max);
        //
        xmin.set(x_min.year());
        xmax.set(x_max.year());
        ymin.set(*y_min);
        ymax.set(*y_max);

        for value in 0..10 {
            let y = 0.95 * canvas.height() as f64
                - (value as f64 / 10.0) * (0.95 * canvas.height() as f64)
                - 0.04 * canvas.height() as f64;
            let _ = ctx.fill_text(
                format!("{:.2}", y_start + value as f64 * (y_end / 10.0)).as_str(),
                0.005 * canvas.width() as f64,
                y,
            );
        }
        ctx.stroke();

        if values.len() > 0 && dates.len() > 0 {
            let max_days = ts
                .iter()
                .max()
                .unwrap_or(&DataPoint {
                    period: NaiveDate::from_ymd_opt(2025, 7, 1).unwrap(),
                    v: 0.0,
                })
                .period
                .signed_duration_since(*x_min)
                .num_days();

            ctx.move_to(
                0.1f64 * canvas.width() as f64,
                0.9f64 * canvas.height() as f64
                    - ((values[0] - *y_min) / y_max) * 0.9 * canvas.height() as f64,
            );

            for dp in &ts {
                let days = dp.period.signed_duration_since(*x_min).num_days();
                let x = 0.1 * canvas.width() as f64
                    + days as f64 / max_days as f64 * 0.85 * canvas.width() as f64;
                let y = 0.9 * canvas.height() as f64
                    - (dp.v - *y_min) / *y_max * 0.9 * canvas.height() as f64;
                ctx.line_to(x, y);
                ctx.stroke();
            }
        }
    });

    rsx! {
        div {class:"flex flex-col justify-start items-center w-[100%] h-[100%]",
            div { class:"w-[100%] h-[100%] relative flex flex-col",
                div { class:"sticky z-50 top-[0px] left-[0px] flex flex-col gap-x-[0.5rem] h-[10%] w-[100%] rounded-t-[0.85rem] bg-[#000]",
                    button {class:"text-[1.5rem] font-bold w-[100%] cursor-pointer", onclick: move |_| series_btn.set(!series_btn()), {format!("{}", if selected_tab() == "annual" && selected_annual_series().as_str() != "" {selected_annual_series()} else if selected_tab() == "quarterly" && selected_quarterly_series().as_str() != "" {selected_quarterly_series()} else {String::from("Time Series Data")}) }}
        div {class:"flex flex-row", for (k, _) in series().into_iter() {
            {
                let kpy = k.clone();
                rsx! {
                    div {class:"duration-[0.5s]", border_bottom: if kpy.to_owned() == selected_tab() {"solid 2px #ffffff"} else { "none"}, class:"w-[100%] h-[100%] flex flex-row justify-center items-center",
                        h2 {class:"my-[0rem]",
                            button {class:"bg-[#000000] text-[#ffffff] border-none text-[1.5rem] cursor-pointer", onclick:move |_| selected_tab.set(kpy.to_owned()) ,
                                "{kpy.to_owned().to_uppercase().to_owned()}"
                            }
                        }
                    }
                }
            }
        }}
    }
        for (k, v) in series().into_iter() {
            div {class:"absolute z-0 top-[1rem] left-[0rem] px-[0.5rem] flex flex-col w-[100%] h-[100%] overflow-y-scroll", visibility: if k.to_owned() == selected_tab() {"visible"} else {"hidden"},

                match v {
                    Value::String(s) => rsx! {
                        p {{String::from(s)}}},
                    Value::Object(o) => {
                        let o = o.clone();

                        rsx! {
                            div {visibility: if series_btn() && k.to_owned() == selected_tab() {"visible"} else {"hidden"}, class:"flex flex-col justify-start items-start bg-[#0009] h-[100%]",
                                div { class:"w-[100%] h-[100%] border-none grid grid-cols-4 gap-y-[0.5rem]",
                                    for (s, _) in o.into_iter() {
                                        {let s = s.clone();
                                        rsx! {
                                            { let y = k.clone();
                                        rsx! {div {class:"flex flex-row m-[0px] w-[100%] h-[90%] gap-x-[1rem] items-center justify-start w-[100%]",
                                            input {r#type:"radio", checked: match selected_tab().as_str() {
                                                "annual" => s == selected_annual_series(),
                                                "quarterly" => s == selected_quarterly_series(),
                                                _ => false
                                            }, onchange:move |_| {
                                                match y.as_str() {
                                                    "annual" => {
                                                        // let mut series = selected_annual_series.write();
                                                        // if series.contains(&s) { series.remove(&s); }
                                                        // else { series.insert(s.to_owned()); }
                                                        selected_annual_series.set(s.to_owned());
                                                    },
                                                    "quarterly" => {
                                                        // let mut series = selected_quarterly_series.write();
                                                        // if series.contains(&s) { series.remove(&s); }
                                                        // else { series.insert(s.to_owned()); }
                                                        selected_quarterly_series.set(s.to_owned());
                                                    },
                                                    _ => ()
                                                }
                                            }}
                                            p {class:"my-[0px]", "{s.to_owned()}"}
                                        }
                                        }}}}

                                        }
                                    }
                                }
                            }
                        }

                    ,
                    _ => rsx! {li {{String::from("Computer says no")}}}
                }
                }
            }

            div {id:"chart-container", class:"w-[100%] h-[90%] flex flex-col items-center justify-center",
                canvas {visibility: match selected_tab().as_str() {
                "annual" => if selected_annual_series() == String::from("") { "hidden"} else {"visible"},
                "quarterly" => if selected_quarterly_series() == String::from("") { "hidden" } else {"visible"},
                _ => "hidden"
            },width:"100%", height:"100%", class:"w-[100%] h-[100%] p-[1rem]", id:"chart"}}
            // p{"xmin: {xmin} xmax: {xmax} ymin: {ymin} ymax:{ymax}"}
            // p{step: "{step}"}
            // p {"{selected_financial:?}"}
    }}
    }
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
