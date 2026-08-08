use bc_indicators_gw::gw::Indicators;
use bc_signals::main_trait::SignalReady;
use bc_signals::prelude::*;
use bc_signals_train_gw::gw::SignalsTrain;
use bc_signals_train_gw::gw::{get_src, get_src_series};
use bc_utils::other::{transpose, vec_len_sync_set};
use bc_utils_lg::structs::settings::{SETTINGS_INDS, SETTINGS_SIGNAL, SETTINGS_SIGNALS};
use bc_utils_lg::traits::w::{w_scan, w_src, w_sum};
use bc_utils_lg::types::maps::{MAP, MAP_LINK, PACK};

pub fn get_signals<'a>(signals: &MAP<&str, Vec<Signal>>, s: &SETTINGS_SIGNAL) -> Vec<Vec<Signal>> {
    let mut res = vec![];
    for used_signal in &s.used_signals {
        res.push(signals[used_signal.as_str()].clone());
    }
    if !res.is_empty() {
        vec_len_sync_set(&mut res);
        return transpose(res);
    }
    Default::default()
}

fn get_signals_series(s: &SETTINGS_SIGNAL, signals: &MAP<&str, Signal>) -> Vec<Signal> {
    let mut signals_arg = vec![];
    for used_signal in &s.used_signals {
        signals_arg.push(signals[used_signal.as_str()]);
    }
    signals_arg
}

#[derive(Default, Clone)]
pub struct Signals<'a>(pub MAP<&'a str, Box<dyn SignalReady>>);

impl W for Signals<'_> {
    fn w(&self) -> usize {
        self.0.values().map(|v| v.w()).max().unwrap_or_default()
    }
}

impl<'a> Signals<'a> {
    pub fn w_map_all(&self, s: &'a SETTINGS_SIGNALS) -> MAP_LINK<&'a str, usize> {
        w_scan(
            self.0.iter(),
            s.iter(),
            |v| v.w(),
            |setting, init, k| {
                [
                    w_src(&setting.used_src),
                    w_sum(&setting.used_signals, init),
                    init[k.as_str()],
                ]
            },
        )
    }
    pub fn w_all(&self, s: &SETTINGS_SIGNALS) -> usize {
        self.w_map_all(s)
            .values()
            .max()
            .copied()
            .unwrap_or_default()
    }
}

impl<'a> Signals<'a> {
    pub fn new_empty_bf(
        s: &'a SETTINGS_SIGNALS,
        pack: &PACK<SETTINGS_SIGNAL, Box<dyn SignalReady>>,
    ) -> Self {
        Signals(
            s.iter()
                .map(|(signal_name, settings_signal)| {
                    let signal = pack[settings_signal.key.as_str()](settings_signal);
                    (signal_name.as_str(), signal)
                })
                .collect(),
        )
    }
    pub fn init_bf(
        &self,
        buffer: &[Vec<f64>],
        s: &'a SETTINGS_SIGNALS,
        s_ind: &'a SETTINGS_INDS,
        s_signals_train: &'a SETTINGS_SIGNALS,
        indicators: &Indicators,
        signals_train: &SignalsTrain,
    ) {
        let indicators = indicators.clone();
        let signals_train = signals_train.clone();
        let buffer_vec_trans = transpose(buffer.to_vec());
        let w = buffer_vec_trans.len() - self.w_all(s);
        let (buffer_init, buffer_vec) = (
            transpose(buffer_vec_trans[..w].to_vec()),
            transpose(buffer_vec_trans[w..].to_vec()),
        );
        if indicators.w() != 0 {
            indicators.init_bf(&buffer_init, s_ind);
        }
        if signals_train.w() != 0 {
            signals_train.init_bf(&buffer_init, s_signals_train, s_ind, &indicators);
        }
        let map_ind = indicators.vec(&buffer_vec, s_ind);
        let map_st = signals_train.vec(&buffer_vec, s_signals_train, &map_ind);
        let mut map_sign = MAP::default();
        for (k, setting) in s.iter() {
            let signal = &self.0[k.as_str()];
            let src = get_src(buffer, &map_ind, &map_st, setting);
            let signals = get_signals(&map_sign, setting);
            signal.init_bf(
                &src.get(..signal.w()).unwrap_or_default(),
                &signals.get(..signal.w()).unwrap_or_default(),
            );
            map_sign.insert(
                k.as_str(),
                signal.signals_vec(
                    &src.get(signal.w()..).unwrap_or_default(),
                    &signals.get(signal.w()..).unwrap_or_default(),
                ),
            );
            signal.init_bf(&src, &signals);
        }
    }
    pub fn new(
        buffer: &[Vec<f64>],
        s: &'a SETTINGS_SIGNALS,
        s_ind: &'a SETTINGS_INDS,
        s_signals_train: &'a SETTINGS_SIGNALS,
        indicators: &Indicators,
        signals_train: &SignalsTrain,
        pack: &PACK<SETTINGS_SIGNAL, Box<dyn SignalReady>>,
    ) -> Self {
        let bind = Signals::new_empty_bf(s, pack);
        bind.init_bf(buffer, s, s_ind, s_signals_train, indicators, signals_train);
        bind
    }
}

impl<'a> Signals<'a> {
    pub fn series(
        &self,
        buffer: &[Vec<f64>],
        s: &'a SETTINGS_SIGNALS,
        indications: &MAP<&str, f64>,
        signals_train: &MAP<&str, f64>,
    ) -> MAP<&'a str, Signal> {
        s.iter().fold(MAP::default(), |mut init, (k, setting)| {
            init.insert(
                k.as_str(),
                self.0[k.as_str()].signal(
                    &get_src_series(buffer, indications, signals_train, setting),
                    &get_signals_series(setting, &init),
                ),
            );
            init
        })
    }
    pub fn execute_bf(&self) {
        for s in self.0.values() {
            s.execute_bf();
        }
    }
    pub fn vec(
        &self,
        buffer: &[Vec<f64>],
        s: &'a SETTINGS_SIGNALS,
        indications: &MAP<&str, Vec<f64>>,
        signals_train: &MAP<&str, Vec<f64>>,
    ) -> MAP<&'a str, Vec<Signal>> {
        s.iter().fold(MAP::default(), |mut init, (k, setting)| {
            let signal = &self.0[k.as_str()];
            init.insert(
                k.as_str(),
                signal.signals_vec(
                    &get_src(buffer, indications, signals_train, setting),
                    &get_signals(&init, setting),
                ),
            );
            init
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;

    use bc_packs::{PACK_IND, PACK_SIGN, PACK_SIGN_TR};
    use bc_signals::{invert::INVERT, th::TH};
    use bc_test_kit::prelude::*;
    use bc_test_kit::settings::signals::SIGNALS;
    use bc_test_kit::settings::signals_train::SIGNALS_TRAIN;

    use pretty_assertions::assert_eq as assert_eq_pr;

    #[test]
    fn new_empty_bf_res_1() {
        let bind = Signals::new_empty_bf(&SIGNALS, &PACK_SIGN);
        assert_eq_pr!(
            (bind.0["th_1"].as_ref() as &dyn Any)
                .downcast_ref::<TH>()
                .unwrap(),
            &TH::new(0.0001, 0.0001, 1., 0, 0, 0, 0., -1., 1.)
        );
    }

    #[test]
    fn w_all_res_1() {
        assert_eq_pr!(
            Signals::new_empty_bf(&SIGNALS, &PACK_SIGN).w_all(&SIGNALS,),
            2
        );
    }

    #[test]
    fn init_bf_res_1() {
        let indicators = Indicators::new_empty_bf(&INDICATIONS, &PACK_IND);
        let signals_train = SignalsTrain::new_empty_bf(&SIGNALS_TRAIN, &PACK_SIGN_TR);
        let signals = Signals::new_empty_bf(&SIGNALS, &PACK_SIGN);
        let w_all = SRC.len() - signals.w_all(&SIGNALS);
        let (buffer_init, buffer_res) = (
            transpose(SRC[..w_all].to_vec()),
            transpose(SRC[w_all..].to_vec()),
        );
        indicators.init_bf(&buffer_init, &INDICATIONS);
        signals_train.init_bf(&buffer_init, &SIGNALS_TRAIN, &INDICATIONS, &indicators);
        let map_ind = indicators.vec(&buffer_res, &INDICATIONS);
        let map_st = signals_train.vec(&buffer_res, &SIGNALS_TRAIN, &map_ind);
        signals.init_bf(
            &SRC_TRANSPOSE,
            &SIGNALS,
            &INDICATIONS,
            &SIGNALS_TRAIN,
            &indicators,
            &signals_train,
        );
        let res = signals.0["th_1"].clone();
        res.init_bf(
            &get_src(&SRC_TRANSPOSE, &map_ind, &map_st, &SIGNALS["th_1"]),
            &[],
        );

        let map_ind_series = indicators.series(&SRC_TRANSPOSE, &INDICATIONS);
        let map_st_series = signals_train.series(&SRC_TRANSPOSE, &SIGNALS_TRAIN, &map_ind_series);
        let series = signals.series(&SRC_TRANSPOSE, &SIGNALS, &map_ind_series, &map_st_series);
        assert_eq_pr!(
            series["th_1"],
            res.signal(
                &get_src_series(
                    &SRC_TRANSPOSE,
                    &map_ind_series,
                    &map_st_series,
                    &SIGNALS["th_1"]
                ),
                &[]
            )
        );
    }

    #[test]
    fn series_res_1() {
        let signals_train = SignalsTrain::new_empty_bf(&SIGNALS_TRAIN, &PACK_SIGN_TR);
        let indicators = Indicators::new_empty_bf(&INDICATIONS, &PACK_IND);
        indicators.init_bf(&SRC_TRANSPOSE, &INDICATIONS);
        signals_train.init_bf(&SRC_TRANSPOSE, &SIGNALS_TRAIN, &INDICATIONS, &indicators);
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
        let res = signals.0["th_1"].signal(&[SRC_EL1[1]], &[]);
        let series = signals.series(&SRC_TRANSPOSE, &SIGNALS, &map_ind, &map_st);
        assert_eq_pr!(series["th_1"], res,);
        assert_eq_pr!(series["invert_1"], INVERT::default().signal(&[], &[res]),);
    }

    #[test]
    fn vec_res_1() {
        let signals_train = SignalsTrain::new_empty_bf(&SIGNALS_TRAIN, &PACK_SIGN_TR);
        let indicators = Indicators::new_empty_bf(&INDICATIONS, &PACK_IND);
        indicators.init_bf(&SRC_TRANSPOSE, &INDICATIONS);
        signals_train.init_bf(&SRC_TRANSPOSE, &SIGNALS_TRAIN, &INDICATIONS, &indicators);
        let map_ind = indicators.vec(&SRC_TRANSPOSE, &INDICATIONS);
        let map_st = signals_train.vec(&SRC_TRANSPOSE, &SIGNALS_TRAIN, &map_ind);
        let signals = Signals::new(
            &SRC_TRANSPOSE,
            &SIGNALS,
            &INDICATIONS,
            &SIGNALS_TRAIN,
            &indicators,
            &signals_train,
            &PACK_SIGN,
        );
        let res = signals.0["th_1"].clone().signals_vec(
            &get_src(&SRC_TRANSPOSE, &map_ind, &map_st, &SIGNALS["th_1"]),
            &[],
        );
        let vec = signals.vec(&SRC_TRANSPOSE, &SIGNALS, &map_ind, &map_st);
        assert_eq_pr!(vec["th_1"], res,);
        assert_eq_pr!(
            vec["invert_1"],
            INVERT::default().signals_vec(&[], &transpose(vec![res])),
        );
    }
}
