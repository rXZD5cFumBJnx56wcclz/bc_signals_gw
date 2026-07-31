use std::hint::black_box;

use bc_packs::{PACK_IND, PACK_SIGN, PACK_SIGN_TR};
use bc_signals_train_gw::gw::SignalsTrain;
use bc_test_kit::prelude::*;
use bc_test_kit::settings::signals::SIGNALS;
use bc_test_kit::settings::signals_train::SIGNALS_TRAIN;
use criterion::{Criterion, criterion_group, criterion_main};

use bc_indicators_gw::gw::Indicators;
use bc_signals_gw::gw::Signals;

fn series_1(c: &mut Criterion) {
    let indicators = Indicators::new(&SRC_TRANSPOSE, &INDICATIONS, &PACK_IND);
    let signals_train = SignalsTrain::new(
        &SRC_TRANSPOSE,
        &SIGNALS_TRAIN,
        &INDICATIONS,
        &indicators,
        &PACK_SIGN_TR,
    );
    let map_ind = indicators.series(&SRC_TRANSPOSE, &INDICATIONS);
    let map_st = signals_train.series(&SRC_TRANSPOSE, &SIGNALS_TRAIN, &map_ind);
    let signals = Signals::new(
        &SRC_TRANSPOSE,
        &SIGNALS,
        &INDICATIONS,
        &SIGNALS_TRAIN,
        &indicators,
        &signals_train,
        &PACK_SIGN,
    );
    c.bench_function("series_1", |b| {
        b.iter(|| {
            signals.series(
                black_box(&SRC_TRANSPOSE),
                black_box(&SIGNALS),
                &map_ind,
                &map_st,
            )
        })
    });
}

criterion_group!(benches, series_1,);
criterion_main!(benches);
