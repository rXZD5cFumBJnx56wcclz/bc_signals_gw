use std::hint::black_box;

use bc_packs::{PACK_IND, PACK_SIGN};
use bc_test_kit::prelude::*;
use bc_utils_lg::structs::settings::{
    SETTINGS_IND, SETTINGS_INDS, SETTINGS_SIGNAL, SETTINGS_SIGNALS, SETTINGS_USED_USIZE,
};
use bc_utils_lg::types::maps::MAP;
use criterion::{Criterion, criterion_group, criterion_main};

use bc_indicators_gw::gw::{Indicators, IndicatorsGateway};
use bc_signals_gw::gw::{Signals, SignalsGateway};

fn get_signals_from_settings_1(c: &mut Criterion) {
    let s = SETTINGS_SIGNALS::from_iter([(
        "th_1".to_string(),
        SETTINGS_SIGNAL {
            key: "th".to_string(),
            used_src: vec![SETTINGS_USED_USIZE {
                index: 1,
                sub_from_last_i: 0,
            }],
            ..Default::default()
        },
    )]);
    let bind = Default::default();
    let bind2 = Default::default();
    let bind3 = Default::default();
    let bind4 = Default::default();
    let bind5 = Default::default();
    let bind6 = Default::default();
    let bind7 = Default::default();
    let sr = Signals::new(
        &s,
        &bind,
        &bind2,
        &PACK_SIGN,
        &SRC_TRANSPOSE,
        &bind6,
        &bind7,
    );
    let bind8 = Default::default();
    let bind9 = Default::default();
    let bind10 = Default::default();
    let sr_gw = SignalsGateway::new(&sr, &bind3, &bind8, &s, &bind4, &bind9);
    c.bench_function("get_signals_from_settings_1", |b| {
        b.iter(|| {
            sr_gw.signals_series(
                black_box(&bind5),
                black_box(&bind10),
                black_box(&SRC_TRANSPOSE),
            )
        })
    });
}

fn get_signals_from_settings_2(c: &mut Criterion) {
    let settings_indicators = SETTINGS_INDS::from_iter([
        (
            "trend_ma_1".to_string(),
            SETTINGS_IND {
                key: "trend_ma".to_string(),
                used_src: vec![SETTINGS_USED_USIZE {
                    index: 1,
                    sub_from_last_i: 0,
                }],
                ..Default::default()
            },
        ),
        (
            "repeat_1".to_string(),
            SETTINGS_IND {
                key: "repeat".to_string(),
                kwargs_f64: MAP::from_iter([("value".to_string(), 1.0)]),
                used_src: vec![SETTINGS_USED_USIZE {
                    index: 1,
                    sub_from_last_i: 0,
                }],
                ..Default::default()
            },
        ),
    ]);
    let settings_signals = SETTINGS_SIGNALS::from_iter([
        (
            "convert_1".to_string(),
            SETTINGS_SIGNAL {
                key: "convert".to_string(),
                used_ind: vec!["trend_ma_1".to_string(), "repeat_1".to_string()],
                ..Default::default()
            },
        ),
        (
            "change_1".to_string(),
            SETTINGS_SIGNAL {
                key: "change_signal".to_string(),
                used_signals: vec!["convert_1".to_string()],
                ..Default::default()
            },
        ),
        (
            "invert_1".to_string(),
            SETTINGS_SIGNAL {
                key: "invert".to_string(),
                used_signals: vec!["change_1".to_string()],
                ..Default::default()
            },
        ),
    ]);
    let settings_signals_train = Default::default();
    let map_signals_train = Default::default();
    let signals_train = Default::default();
    let indicators = Indicators::new(&settings_indicators, &PACK_IND, &SRC_TRANSPOSE);
    let indicators_gw = IndicatorsGateway::new(&indicators, &settings_indicators);
    let indications = indicators_gw.indications_series(&SRC_TRANSPOSE);
    let signals = Signals::new(
        &settings_signals,
        &settings_signals_train,
        &settings_indicators,
        &PACK_SIGN,
        &SRC_TRANSPOSE,
        &map_signals_train,
        &indicators.indicators_without_bf,
    );
    let signals_gw = SignalsGateway::new(
        &signals,
        &signals_train,
        &indicators,
        &settings_signals,
        &settings_signals_train,
        &settings_indicators,
    );
    let bind1 = Default::default();
    c.bench_function("get_signals_from_settings_2", |b| {
        b.iter(|| {
            signals_gw.signals_series(
                black_box(&indications),
                black_box(&bind1),
                black_box(&SRC_TRANSPOSE),
            )
        })
    });
}

criterion_group!(
    benches,
    get_signals_from_settings_1,
    get_signals_from_settings_2
);
criterion_main!(benches);
