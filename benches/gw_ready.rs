use std::hint::black_box;

use bc_pack_indicators::FUNCS_EXTRACT_ARGS as FUNCS_EXTRACT_ARGS_IND;
use bc_pack_signals_ready::FUNCS_EXTRACT_ARGS as FUNCS_EXTRACT_ARGS_SR;
use bc_utils_lg::statics::prices::SRC_TRANSPOSE;
use bc_utils_lg::structs::settings::{
    SETTINGS_IND, SETTINGS_INDS, SETTINGS_SIGNAL, SETTINGS_SIGNALS, SETTINGS_USED_SRC,
};
use bc_utils_lg::types::maps::MAP;
use criterion::{Criterion, criterion_group, criterion_main};

use bc_indicators_gw::gw::{Indicators, IndicatorsGateway};
use bc_signals_gw::gw_ready::{SignalsReady, SignalsReadyGateway};

fn get_signals_ready_from_settings_1(c: &mut Criterion) {
    let s = SETTINGS_SIGNALS::from_iter([(
        "th_1".to_string(),
        SETTINGS_SIGNAL {
            key: "th".to_string(),
            used_src: vec![SETTINGS_USED_SRC { index: 1, sub_from_last_i: 0 }],
            ..Default::default()
        },
    )]);
    let bind = Default::default();
    let bind2 = Default::default();
    let bind3 = Default::default();
    let bind4 = Default::default();
    let bind5 = Default::default();
    let sr = SignalsReady::new(&s, &bind, &FUNCS_EXTRACT_ARGS_SR(), &SRC_TRANSPOSE, &bind2);
    let sr_gw = SignalsReadyGateway::new(&sr, &bind3, &s, &bind4);
    c.bench_function("get_signals_ready_from_settings_1", |b| {
        b.iter(|| sr_gw.signals_series(black_box(&bind5), black_box(&SRC_TRANSPOSE)))
    });
}

fn get_signals_ready_from_settings_2(c: &mut Criterion) {
    let settings_indicators = SETTINGS_INDS::from_iter([
        (
            "trend_ma_1".to_string(),
            SETTINGS_IND {
                key: "trend_ma".to_string(),
                used_src: vec![SETTINGS_USED_SRC { index: 1, sub_from_last_i: 0 }],
                ..Default::default()
            },
        ),
        (
            "repeat_1".to_string(),
            SETTINGS_IND {
                key: "repeat".to_string(),
                kwargs_f64: MAP::from_iter([("value".to_string(), 1.0)]),
                used_src: vec![SETTINGS_USED_SRC { index: 1, sub_from_last_i: 0 }],
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
    let indicators = Indicators::new(
        &settings_indicators,
        &FUNCS_EXTRACT_ARGS_IND(),
        &SRC_TRANSPOSE,
    );
    let indicators_gw = IndicatorsGateway::new(&indicators, &settings_indicators);
    let indications = indicators_gw.indications_series(&SRC_TRANSPOSE);
    let signals = SignalsReady::new(
        &settings_signals,
        &settings_indicators,
        &FUNCS_EXTRACT_ARGS_SR(),
        &SRC_TRANSPOSE,
        &indicators.indicators_without_bf,
    );
    let signals_gw = SignalsReadyGateway::new(
        &signals,
        &indicators,
        &settings_signals,
        &settings_indicators,
    );
    c.bench_function("get_signals_ready_from_settings_2", |b| {
        b.iter(|| signals_gw.signals_series(black_box(&indications), black_box(&SRC_TRANSPOSE)))
    });
}

criterion_group!(
    benches,
    get_signals_ready_from_settings_1,
    get_signals_ready_from_settings_2
);
criterion_main!(benches);
