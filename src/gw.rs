use bc_indicators::prelude::Indicator;
use bc_indicators_gw::gw::Indicators;
use bc_signals::main_trait::SignalReady;
use bc_signals::prelude::*;
use bc_signals_train::main_trait::SignalTrain;
use bc_signals_train_gw::gw::SignalsTrain;
use bc_signals_train_gw::gw::get_src as get_src_signals_train;
use bc_utils::other::{procedure_used, transpose, vec_len_sync_set};
use bc_utils_lg::structs::settings::{SETTINGS_INDS, SETTINGS_SIGNAL, SETTINGS_SIGNALS};
use bc_utils_lg::types::maps::{MAP, PACK};

pub fn get_signals<'a>(
    used_signals: &Vec<String>,
    s_signals: &SETTINGS_SIGNALS,
    s_signals_train: &SETTINGS_SIGNALS,
    s_inds: &SETTINGS_INDS,
    src_transpose: &[Vec<f64>],
    map_signals: &MAP<&'a str, Box<dyn SignalReady>>,
    map_signals_train: &MAP<&'a str, Box<dyn SignalTrain>>,
    map_indicators: &MAP<&'a str, Box<dyn Indicator>>,
) -> Vec<Vec<Signal>> {
    let mut res = vec![];
    for used_signal in used_signals {
        res.push(map_signals[used_signal.as_str()].signals_vec(
            &get_src_signals_train(
                &s_signals[used_signal],
                s_inds,
                s_signals_train,
                src_transpose,
                map_indicators,
                map_signals_train,
            ),
            &get_signals(
                &s_signals[used_signal].used_signals,
                s_signals,
                s_signals_train,
                s_inds,
                src_transpose,
                map_signals,
                map_signals_train,
                map_indicators,
            ),
        ));
    }
    if !res.is_empty() {
        vec_len_sync_set(&mut res);
        return transpose(res);
    }
    Default::default()
}

fn get_signals_series(s: &SETTINGS_SIGNAL, signals: &MAP<&str, Signal>) -> Vec<Signal> {
    let mut signals_arg = vec![];
    for signals_arg_el in &s.used_signals {
        signals_arg.push(signals[signals_arg_el.as_str()].clone());
    }
    signals_arg
}

fn get_src_series(
    s: &SETTINGS_SIGNAL,
    src_transpose: &[Vec<f64>],
    indications: &MAP<&str, f64>,
    signals_train: &MAP<&str, f64>,
) -> Vec<f64> {
    let mut res = vec![];
    for src_arg_el in &s.used_src {
        res.push({
            let sk = &src_transpose[src_arg_el.index];
            sk[sk.len() - 1 - src_arg_el.sub_from_last_i]
        });
    }
    for ind_arg_el in &s.used_ind {
        res.push(indications[ind_arg_el.as_str()]);
    }
    for signals_train_used in &s.used_signals_train {
        res.push(signals_train[signals_train_used.as_str()]);
    }
    if !s.procedure_used_src.is_empty() {
        res = procedure_used(res, &s.procedure_used_src);
    }
    res
}

pub struct Signals<'a> (pub MAP<&'a str, Box<dyn SignalReady>>);

pub trait SignalsExt<'a> {
    fn new_empty_bf(
        s_signals: &'a SETTINGS_SIGNALS,
        pack: &PACK<SETTINGS_SIGNAL, Box<dyn SignalReady>>,
    ) -> Self;
    fn init_bf(
        &self,
        s_signals: &'a SETTINGS_SIGNALS,
        s_signals_train: &'a SETTINGS_SIGNALS,
        s_inds: &'a SETTINGS_INDS,
        buffer: &[Vec<f64>],
        signals_train: &SignalsTrain,
        indicators: &Indicators,
    );
    fn new(
        s_signals: &'a SETTINGS_SIGNALS,
        s_signals_train: &'a SETTINGS_SIGNALS,
        s_inds: &'a SETTINGS_INDS,
        buffer: &[Vec<f64>],
        signals_train: &SignalsTrain,
        indicators: &Indicators,
        pack: &PACK<SETTINGS_SIGNAL, Box<dyn SignalReady>>,
    ) -> Self;
}

impl<'a> SignalsExt<'a> for Signals<'a> {
    fn new_empty_bf(
        s_signals: &'a SETTINGS_SIGNALS,
        pack: &PACK<SETTINGS_SIGNAL, Box<dyn SignalReady>>,
    ) -> Self {
        Signals(s_signals
            .iter()
            .map(|(signal_name, settings_signal)| {
                let signal = pack[settings_signal.key.as_str()](settings_signal);
                (signal_name.as_str(), signal)
            })
            .collect())
    }
    fn init_bf(
        &self,
        s_signals: &'a SETTINGS_SIGNALS,
        s_signals_train: &'a SETTINGS_SIGNALS,
        s_inds: &'a SETTINGS_INDS,
        buffer: &[Vec<f64>],
        signals_train: &SignalsTrain,
        indicators: &Indicators,
    ) {
        let signals_train = signals_train.0.clone();
        let indicators = indicators.0.clone();
        let signals = self.0.clone();
        for (k, s) in s_signals {
            self.0[k.as_str()].init_bf(
                &get_src_signals_train(
                    s,
                    s_inds,
                    s_signals_train,
                    buffer,
                    &indicators,
                    &signals_train,
                ),
                &get_signals(
                    &s.used_signals,
                    s_signals,
                    s_signals_train,
                    s_inds,
                    buffer,
                    &signals,
                    &signals_train,
                    &indicators,
                ),
            );
        }
    }
    fn new(
        s_signals: &'a SETTINGS_SIGNALS,
        s_signals_train: &'a SETTINGS_SIGNALS,
        s_inds: &'a SETTINGS_INDS,
        buffer: &[Vec<f64>],
        signals_train: &SignalsTrain,
        indicators: &Indicators,
        pack: &PACK<SETTINGS_SIGNAL, Box<dyn SignalReady>>,
    ) -> Self {
        let bind = Signals::new_empty_bf(s_signals, pack);
        bind.init_bf(
            s_signals,
            s_signals_train,
            s_inds,
            buffer,
            signals_train,
            indicators,
        );
        bind
    }
}

impl W for Signals<'_> {
    fn w(&self) -> usize {
        self.0.values().map(|v| v.w()).max().unwrap()
    }
}

#[derive(Default)]
pub struct SignalsGateway<'a> {
    pub signals: *const Signals<'a>,
    pub signals_train: *const SignalsTrain<'a>,
    pub indicators: *const Indicators<'a>,
    pub settings_signals_train: *const SETTINGS_SIGNALS,
    pub settings_signals: *const SETTINGS_SIGNALS,
    pub settings_indicators: *const SETTINGS_INDS,
}

impl<'a> SignalsGateway<'a> {
    pub fn new(
        signals: *const Signals<'a>,
        signals_train: *const SignalsTrain<'a>,
        indicators: *const Indicators<'a>,
        settings_signals_train: *const SETTINGS_SIGNALS,
        settings_signals: *const SETTINGS_SIGNALS,
        settings_indicators: *const SETTINGS_INDS,
    ) -> Self {
        Self {
            signals,
            signals_train,
            indicators,
            settings_signals_train,
            settings_signals,
            settings_indicators,
        }
    }
}
impl<'a> SignalsGateway<'a> {
    pub fn signals_series(
        &self,
        indications: &MAP<&str, f64>,
        signals_train: &MAP<&str, f64>,
        src_transpose: &[Vec<f64>],
    ) -> MAP<&'a str, Signal> {
        unsafe { &*self.settings_signals }
            .iter()
            .fold(MAP::default(), |mut map, setting| {
                let key_uniq_str = setting.0.as_str();
                map.insert(
                    key_uniq_str,
                    unsafe { &*self.signals }.0[key_uniq_str].signal_with_bf(
                        &get_src_series(&setting.1, src_transpose, indications, signals_train),
                        &get_signals_series(&setting.1, &map),
                    ),
                );
                map
            })
    }
    pub fn execute_bf(&self) {
        for s in unsafe { &*self.signals }.0.values() {
            s.execute_bf();
        }
    }
    pub fn signals_vec(&self, src_transpose: &[Vec<f64>]) -> MAP<&'a str, Vec<Signal>> {
        unsafe { &*self.settings_signals }
            .iter()
            .map(|(k, setting)| {
                let key_uniq = k.as_str();
                let signal = &unsafe { &*self.signals }.0[key_uniq];
                (
                    key_uniq,
                    signal.signals_vec(
                        &get_src_signals_train(
                            setting,
                            unsafe { &*self.settings_indicators },
                            unsafe { &*self.settings_signals_train },
                            src_transpose,
                            &unsafe { &*self.indicators}.0,
                            &unsafe { &*self.signals_train}.0,
                        ),
                        &get_signals(
                            &setting.used_signals,
                            unsafe { &*self.settings_signals },
                            unsafe { &*self.settings_signals_train },
                            unsafe { &*self.settings_indicators },
                            src_transpose,
                            &unsafe { &*self.signals }.0,
                            &unsafe { &*self.signals_train }.0,
                            &unsafe { &*self.indicators }.0,
                        ),
                    ),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;

    use bc_indicators::{repeat::REPEAT, trend_ma::TREND_MA};
    use bc_packs::{PACK_IND, PACK_SIGN, PACK_SIGN_TR};
    use bc_signals::{change_signal::CHANGE_SIGNAL, convert::CONVERT, invert::INVERT, th::TH};
    use bc_signals_train_gw::gw::{SignalsTrainExt, SignalsTrainGateway};
    use bc_test_kit::prelude::*;
    use bc_utils_lg::structs::settings::{
        SETTINGS_IND, SETTINGS_INDS, SETTINGS_SIGNAL, SETTINGS_SIGNALS, SETTINGS_USED_USIZE,
    };
    use bc_utils_lg::types::maps::MAP;
    use pretty_assertions::{assert_eq as assert_eq_pr, assert_ne as assert_ne_pr};

    use bc_indicators_gw::gw::{IndicatorsExt, IndicatorsGateway};

    #[test]
    fn signals_from_settings_without_bf_res_1() {
        let settings = SETTINGS_SIGNALS::from_iter([(
            "th_1".to_string(),
            SETTINGS_SIGNAL {
                key: "th".to_string(),
                ..Default::default()
            },
        )]);
        let res = Signals::new_empty_bf(&settings, &PACK_SIGN);
        let res_1 = res.0.get("th_1").unwrap().as_ref();
        let rsi_test_1 = TH::default();
        let rsi_test_2 = (res_1 as &dyn Any).downcast_ref::<TH>().unwrap();
        assert_eq_pr!(&rsi_test_1, rsi_test_2);
    }

    #[test]
    fn signals_res_1() {
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
        let buffer = &transpose(transpose(SRC_TRANSPOSE.to_vec())[..49].to_vec());
        let settings_signals_train = Default::default();
        let indicators = Indicators::new(&buffer, &settings_indicators, &PACK_IND);
        let signals_train = SignalsTrain::new(
            &settings_signals_train,
            &settings_indicators,
            &buffer,
            &indicators,
            &PACK_SIGN_TR,
        );
        let signals = Signals::new(
            &settings_signals,
            &settings_signals_train,
            &settings_indicators,
            &buffer,
            &signals_train,
            &indicators,
            &PACK_SIGN,
        );
        let indicators_gw = IndicatorsGateway::new(&indicators, &settings_indicators);
        let signals_train_gw = SignalsTrainGateway::new(
            &signals_train,
            &indicators,
            &settings_signals_train,
            &settings_indicators,
        );
        let indications = indicators_gw.indications_series(&SRC_TRANSPOSE);
        let signals_train_ = signals_train_gw.signals_series(&indications, &SRC_TRANSPOSE);
        let signals_gw = SignalsGateway::new(
            &signals,
            &signals_train,
            &indicators,
            &settings_signals_train,
            &settings_signals,
            &settings_indicators,
        );
        let res_1 =
            signals_gw.signals_series(&indications, &signals_train_, &SRC_TRANSPOSE)["invert_1"];
        let res_2 = INVERT::default().signal(
            &vec![],
            &vec![vec![
                CHANGE_SIGNAL::default().signal(
                    &vec![],
                    &CONVERT
                        .signals_vec(
                            &TREND_MA::default()
                                .ind_vec(
                                    &SRC.iter()
                                        .map(|v| v[1..].to_vec())
                                        .collect::<Vec<Vec<f64>>>(),
                                )
                                .into_iter()
                                .zip(REPEAT::new(1.0).ind_vec(&SRC))
                                .map(|(v1, v2)| vec![v1, v2])
                                .collect::<Vec<Vec<f64>>>(),
                            &vec![],
                        )
                        .into_iter()
                        .map(|s| vec![s])
                        .collect::<Vec<Vec<Signal>>>(),
                ),
            ]],
        );
        assert_eq_pr!(res_1, res_2);
    }

    #[test]
    fn signals_execute_bf_res_1() {
        let settings_signals = SETTINGS_SIGNALS::from_iter([(
            "th_1".to_string(),
            SETTINGS_SIGNAL {
                key: "th".to_string(),
                used_src: vec![SETTINGS_USED_USIZE {
                    index: 1,
                    ..Default::default()
                }],
                kwargs_f64: MAP::from_iter([
                    ("th_min".to_string(), 0.00000001),
                    ("th_max".to_string(), 0.00000001),
                ]),
                ..Default::default()
            },
        )]);
        let s_inds = Default::default();
        let s_sign_train = Default::default();
        let indicators = Default::default();
        let signals_train = Default::default();
        let signals = Signals::new(
            &settings_signals,
            &s_sign_train,
            &s_inds,
            &transpose(SRC[..49].to_vec()),
            &signals_train,
            &indicators,
            &PACK_SIGN,
        );
        let signals_gw = SignalsGateway::new(
            &signals,
            &signals_train,
            &indicators,
            &s_sign_train,
            &settings_signals,
            &s_inds,
        );
        let indications = Default::default();
        let signals_train = Default::default();
        assert_eq_pr!(
            signals_gw.signals_series(&indications, &signals_train, &SRC_TRANSPOSE),
            signals_gw.signals_series(&indications, &signals_train, &SRC_TRANSPOSE),
        );
        assert_ne_pr!(
            {
                let bind = signals_gw.signals_series(&indications, &signals_train, &SRC_TRANSPOSE);
                signals_gw.execute_bf();
                bind
            },
            {
                let bind = signals_gw.signals_series(&indications, &signals_train, &SRC_TRANSPOSE);
                signals_gw.execute_bf();
                bind
            },
        );
    }

    #[test]
    fn signals_vec_res_1() {
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
        let indicators = Indicators::new(&SRC_TRANSPOSE, &settings_indicators, &PACK_IND);
        let signals_train = SignalsTrain::new(
            &settings_signals_train,
            &settings_indicators,
            &SRC_TRANSPOSE,
            &indicators,
            &PACK_SIGN_TR,
        );
        let signals = Signals::new(
            &settings_signals,
            &settings_signals_train,
            &settings_indicators,
            &SRC_TRANSPOSE,
            &signals_train,
            &indicators,
            &PACK_SIGN,
        );
        let signals_gw = SignalsGateway::new(
            &signals,
            &signals_train,
            &indicators,
            &settings_signals_train,
            &settings_signals,
            &settings_indicators,
        );
        let res_1 = &signals_gw.signals_vec(&SRC_TRANSPOSE)["invert_1"];
        let res_2 = &INVERT::default().signals_vec(
            &vec![],
            &CHANGE_SIGNAL::default()
                .signals_vec(
                    &vec![],
                    &CONVERT
                        .signals_vec(
                            &TREND_MA::default()
                                .ind_vec(
                                    &SRC.iter()
                                        .map(|v| v[1..].to_vec())
                                        .collect::<Vec<Vec<f64>>>(),
                                )
                                .into_iter()
                                .zip(REPEAT::new(1.0).ind_vec(&SRC))
                                .map(|(v1, v2)| vec![v1, v2])
                                .collect::<Vec<Vec<f64>>>(),
                            &vec![],
                        )
                        .into_iter()
                        .map(|s| vec![s])
                        .collect::<Vec<Vec<Signal>>>(),
                )
                .into_iter()
                .map(|s| vec![s])
                .collect::<Vec<Vec<Signal>>>(),
        );
        assert_eq_pr!(
            res_1
                .iter()
                .filter(|s| !s.signal.is_nan())
                .collect::<Vec<_>>(),
            res_2
                .iter()
                .filter(|s| !s.signal.is_nan())
                .collect::<Vec<_>>()
        );
    }
}
