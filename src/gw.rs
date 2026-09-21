use bc_gw_utils::prelude::*;
use bc_indicators_gw::gw::Indicators;
use bc_signals::prelude::*;
use bc_signals_train_gw::gw::*;
use bc_utils::other::transpose;
use bc_utils_lg::prelude::*;

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
    pub fn init_empty(
        &mut self,
        s: &'a SETTINGS_SIGNALS,
        pack: &PACK<SETTINGS_SIGNAL, Box<dyn SignalReady>>,
    ) {
        *self = Signals(
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
            let mut src = SrcGw::default();
            src.push_vec(buffer, &setting.used_src);
            src.push_map(&map_ind, &setting.used_ind);
            src.push_map(&map_st, &setting.used_signals_train);
            src.all_check(&setting.procedure_used_src);
            let mut signals = SignalsGw::default();
            signals.push(&map_sign, &setting.used_signals);
            signals.all_check();
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
    pub fn init(
        &mut self,
        buffer: &[Vec<f64>],
        s: &'a SETTINGS_SIGNALS,
        s_ind: &'a SETTINGS_INDS,
        s_signals_train: &'a SETTINGS_SIGNALS,
        indicators: &Indicators,
        signals_train: &SignalsTrain,
        pack: &PACK<SETTINGS_SIGNAL, Box<dyn SignalReady>>,
    ) {
        self.init_empty(s, pack);
        self.init_bf(buffer, s, s_ind, s_signals_train, indicators, signals_train);
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
            let mut src = SrcGwSeries::default();
            src.push_vec(buffer, &setting.used_src);
            src.push_map(&indications, &setting.used_ind);
            src.push_map(&signals_train, &setting.used_signals_train);
            src.all_check(&setting.procedure_used_src);
            let mut signals = SignalsGwSeries::default();
            signals.push(&init, &setting.used_signals);
            init.insert(k.as_str(), self.0[k.as_str()].signal(&src, &signals));
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
            let mut src = SrcGw::default();
            src.push_vec(buffer, &setting.used_src);
            src.push_map(&indications, &setting.used_ind);
            src.push_map(&signals_train, &setting.used_signals_train);
            src.all_check(&setting.procedure_used_src);
            let mut signals = SignalsGw::default();
            signals.push(&init, &setting.used_signals);
            signals.all_check();
            init.insert(k.as_str(), signal.signals_vec(&src, &signals));
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
    use bc_utils_lg::test_state::prelude::*;

    #[test]
    fn new_empty_bf_res_1() {
        let mut bind = Signals::default();
        bind.init_empty(&SIGNALS, &PACK_SIGN);
        assert_eq_pr!(
            (bind.0["th_1"].as_ref() as &dyn Any)
                .downcast_ref::<TH>()
                .unwrap(),
            &TH::new(0.0001, 0.0001, 1., 0, 0, 0, 0., -1., 1.)
        );
    }

    #[test]
    fn w_all_res_1() {
        let mut bind = Signals::default();
        bind.init_empty(&SIGNALS, &PACK_SIGN);
        assert_eq_pr!(bind.w_all(&SIGNALS,), 2);
    }

    #[test]
    fn init_bf_res_1() {
        let mut indicators = Indicators::default();
        indicators.init_empty(&INDICATIONS, &PACK_IND);
        let mut signals_train = SignalsTrain::default();
        signals_train.init_empty(&SIGNALS_TRAIN, &PACK_SIGN_TR);
        let mut signals = Signals::default();
        signals.init_empty(&SIGNALS, &PACK_SIGN);
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
        let mut src = SrcGw::default();
        src.push_vec(&SRC_TRANSPOSE, &SIGNALS["th_1"].used_src);
        src.push_map(&map_ind, &SIGNALS["th_1"].used_ind);
        src.push_map(&map_st, &SIGNALS["th_1"].used_signals_train);
        src.all_check(&SIGNALS["th_1"].procedure_used_src);
        res.init_bf(&src, &[]);

        let map_ind_series = indicators.series(&SRC_TRANSPOSE, &INDICATIONS);
        let map_st_series = signals_train.series(&SRC_TRANSPOSE, &SIGNALS_TRAIN, &map_ind_series);
        let series = signals.series(&SRC_TRANSPOSE, &SIGNALS, &map_ind_series, &map_st_series);
        let mut src_series = SrcGwSeries::default();
        src_series.push_vec(&SRC_TRANSPOSE, &SIGNALS["th_1"].used_src);
        src_series.push_map(&map_ind_series, &SIGNALS["th_1"].used_ind);
        src_series.push_map(&map_st_series, &SIGNALS["th_1"].used_signals_train);
        src_series.all_check(&SIGNALS["th_1"].procedure_used_src);
        assert_eq_pr!(series["th_1"], res.signal(&src_series, &[]));
    }

    #[test]
    fn series_res_1() {
        let mut indicators = Indicators::default();
        indicators.init_empty(&INDICATIONS, &PACK_IND);
        let mut signals_train = SignalsTrain::default();
        signals_train.init_empty(&SIGNALS_TRAIN, &PACK_SIGN_TR);
        indicators.init_bf(&SRC_TRANSPOSE, &INDICATIONS);
        signals_train.init_bf(&SRC_TRANSPOSE, &SIGNALS_TRAIN, &INDICATIONS, &indicators);
        let map_ind = indicators.series(&SRC_TRANSPOSE, &INDICATIONS);
        let map_st = signals_train.series(&SRC_TRANSPOSE, &SIGNALS_TRAIN, &map_ind);
        let mut signals = Signals::default();
        signals.init(
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
        let mut indicators = Indicators::default();
        indicators.init_empty(&INDICATIONS, &PACK_IND);
        let mut signals_train = SignalsTrain::default();
        signals_train.init_empty(&SIGNALS_TRAIN, &PACK_SIGN_TR);
        indicators.init_bf(&SRC_TRANSPOSE, &INDICATIONS);
        signals_train.init_bf(&SRC_TRANSPOSE, &SIGNALS_TRAIN, &INDICATIONS, &indicators);
        let map_ind = indicators.vec(&SRC_TRANSPOSE, &INDICATIONS);
        let map_st = signals_train.vec(&SRC_TRANSPOSE, &SIGNALS_TRAIN, &map_ind);
        let mut signals = Signals::default();
        signals.init(
            &SRC_TRANSPOSE,
            &SIGNALS,
            &INDICATIONS,
            &SIGNALS_TRAIN,
            &indicators,
            &signals_train,
            &PACK_SIGN,
        );
        let mut src = SrcGw::default();
        src.push_vec(&SRC_TRANSPOSE, &SIGNALS["th_1"].used_src);
        src.push_map(&map_ind, &SIGNALS["th_1"].used_ind);
        src.push_map(&map_st, &SIGNALS["th_1"].used_signals_train);
        src.all_check(&SIGNALS["th_1"].procedure_used_src);
        let res = signals.0["th_1"].clone().signals_vec(&src, &[]);
        let vec = signals.vec(&SRC_TRANSPOSE, &SIGNALS, &map_ind, &map_st);
        assert_eq_pr!(vec["th_1"], res,);
        assert_eq_pr!(
            vec["invert_1"],
            INVERT::default().signals_vec(&[], &transpose(vec![res])),
        );
    }
}
