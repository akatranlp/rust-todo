use std::{any, fmt::format, time};

use anyhow::{Context, bail};
use valhalla::{Actor, Config, LatLon, proto};

const ANDORRA_CONFIG: &str = "valhalla-data/valhalla.json";
const ANDORRA_TEST_LOC_1: LatLon = LatLon(42.50107335756198, 1.510341967860551); // Sant Julia de Loria
const ANDORRA_TEST_LOC_2: LatLon = LatLon(42.50627089323736, 1.521734167223563); // Andorra la Vella

fn route_request(actor: &mut Actor) -> anyhow::Result<String> {
    let request = proto::Options {
        format: proto::options::Format::Json.into(),
        costing_type: proto::costing::Type::Auto.into(),
        locations: vec![
            proto::Location {
                ll: ANDORRA_TEST_LOC_1.into(),
                ..Default::default()
            },
            proto::Location {
                ll: ANDORRA_TEST_LOC_2.into(),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let response = actor.route(&request).context("actor request failed!")?;

    let data = match response {
        valhalla::Response::Json(data) => data,
        _ => {
            println!("{:?}", response);
            bail!("unkown type");
        }
    };
    Ok(data)
}

fn matrix_request(actor: &mut Actor) -> anyhow::Result<String> {
    let request = proto::Options {
        costing_type: proto::costing::Type::Auto.into(),
        sources: vec![
            proto::Location {
                ll: ANDORRA_TEST_LOC_1.into(),
                ..Default::default()
            },
            proto::Location {
                ll: ANDORRA_TEST_LOC_2.into(),
                ..Default::default()
            },
        ],
        targets: vec![
            proto::Location {
                ll: ANDORRA_TEST_LOC_1.into(),
                ..Default::default()
            },
            proto::Location {
                ll: ANDORRA_TEST_LOC_2.into(),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let data = match actor.matrix(&request).context("matrix call failed!")? {
        valhalla::Response::Json(data) => data,
        response => {
            println!("{:?}", response);
            bail!("unkown type");
        }
    };

    Ok(data)
}

pub fn solve() -> anyhow::Result<String> {
    let config = Config::from_file(ANDORRA_CONFIG).context("couldn't get the valhalla config")?;
    let mut actor = Actor::new(&config).context("could not create a new actor")?;
    let route_data = route_request(&mut actor).context("failed to route")?;
    let matrix_data = matrix_request(&mut actor).context("failed to matrix")?;

    Ok(format!("[{},{}]", route_data, matrix_data))
}
