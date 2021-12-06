use poem::Endpoint;
use poem::Request;
use poem::Response;
use poem::endpoint::make_sync;


use serde_json::Value;

pub fn add_base(base: impl Into<String>, spec: &String) -> String {
    let base: String = base.into();

    let mut spec_value = serde_json::from_str::<Value>(spec.as_str()).unwrap();

    if let Value::Object(props) = &mut spec_value {
        if let Some(paths_inner) = props.get_mut("paths") {

            if let Value::Object(props2) = paths_inner {

                let mut new_map = serde_json::Map::new();
                let base = &base;

                for (key, item) in props2.iter() {
                    let new_name = format!("{base}{key}");
                    new_map.insert(new_name, item.clone());
                }

                *props2 = new_map;

            } else {
                panic!("dddd1..");
            }

        } else {
            panic!("dddd2..");
        }

    } else {
        panic!("ddddd..");
    }

    serde_json::to_string(&spec_value).unwrap()
}


/// ```
// pub fn make_sync<F, R>(f: F) -> impl Endpoint<Output = R>
// where
//     F: Fn(Request) -> R + Send + Sync,
//     R: IntoResponse,


pub fn spec_endpoint(base: impl Into<String>, spec: &String) -> impl Endpoint<Output = Response> {
    let spec = add_base(base, &spec);

    make_sync(move |_| {
        Response::builder()
            .content_type("application/json")
            .body(spec.clone())
    })
}



// let spec_endpoint = {
//     let spec = api_service1.spec();
//     let spec = add_base("/api1", &spec);

//     make_sync(move |_| {
//         Response::builder()
//             .content_type("application/json")
//             .body(spec.clone())
//     })
// };

// let endpoint_async = make({
//     let spec = api_service1.spec();
//     let spec = add_base("/api1", &spec);

//     move |_| {
//         let spec = spec.clone();
//         async move {
//             Response::builder()
//                     .content_type("application/json")
//                     .body(spec.clone())
//         }
//     }
// });

