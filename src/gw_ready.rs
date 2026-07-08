use bc_indicators::prelude::Indicator;
use bc_signals::ready::main_trait::SignalReady;
use bc_signals::ready::prelude::*;
use bc_utils_lg::{
    structs::settings::SETTINGS,
    types::maps::{FUNCS_EXTRACT_ARGS_TYPE, MAP},
};

use bc_indicators_gw::gw::{Indicators, get_in_from_settings};
use bc_utils_lg::structs::settings::{SETTINGS_INDS, SETTINGS_SIGNAL, SETTINGS_SIGNALS};

pub fn get_signals_arg_from_settings<'a>(
    used_signals: &Vec<String>,
    procedure_used_signals: &Vec<usize>,
    settings_signals: &SETTINGS_SIGNALS,
    settings_indicators: &SETTINGS_INDS,
    src_transpose: &[Vec<f64>],
    map_signals: &MAP<&'a str, Box<dyn SignalReady>>,
    map_indicators: &MAP<&'a str, Box<dyn Indicator>>,
) -> Vec<Vec<Signal>> {
    let mut res = vec![];
    for used_signal in used_signals {
        res.push(map_signals[used_signal.as_str()].signals_vec(
            &get_in_from_settings(
                &settings_signals[used_signal].used_ind,
                &settings_signals[used_signal].used_src,
                &settings_signals[used_signal].procedure_used_src,
                settings_indicators,
                src_transpose,
                map_indicators,
            ),
            &get_signals_arg_from_settings(
                &settings_signals[used_signal].used_signals,
                &settings_signals[used_signal].procedure_used_signals,
                settings_signals,
                settings_indicators,
                src_transpose,
                map_signals,
                map_indicators,
            ),
        ));
    }
    if !procedure_used_signals.is_empty() {
        res = procedure_used_signals
            .iter()
            .map(|i| res[*i].clone())
            .collect();
    }
    if !res.is_empty() {
        let min_len = res
            .iter()
            .map(|v| v.len())
            .min()
            .expect("this is nan or wtf");
        res = res
            .into_iter()
            .map(|v| v[v.len() - min_len..].to_vec())
            .collect::<Vec<Vec<Signal>>>();
        return (0..min_len)
            .map(|i| res.iter().map(|v1| v1[i].clone()).collect::<Vec<Signal>>())
            .collect::<Vec<Vec<Signal>>>();
    }
    Default::default()
}

pub fn get_signals_from_settings_without_bf<'a>(
    settings: &'a SETTINGS_SIGNALS,
    funcs_extract_args: &MAP<&'a str, fn(&SETTINGS_SIGNAL) -> Box<dyn SignalReady>>,
) -> MAP<&'a str, Box<dyn SignalReady>> {
    settings
        .iter()
        .map(|(signal_name, settings_signal)| {
            let signal = funcs_extract_args[settings_signal.key.as_str()](settings_signal);
            (signal_name.as_str(), signal)
        })
        .collect()
}

pub fn get_signals_from_settings<'a>(
    settings_signals: &'a SETTINGS_SIGNALS,
    settings_indicators: &'a SETTINGS_INDS,
    funcs_extract_args: &MAP<&'a str, fn(&SETTINGS_SIGNAL) -> Box<dyn SignalReady>>,
    src_transpose: &[Vec<f64>],
    map_signals: &MAP<&'a str, Box<dyn SignalReady>>,
    map_indicators: &MAP<&'a str, Box<dyn Indicator>>,
) -> MAP<&'a str, (BF_SIGNALS<'a>, Box<dyn SignalReady>)> {
    settings_signals
        .iter()
        .map(|(signal_name, settings_signal)| {
            let signal = funcs_extract_args[settings_signal.key.as_str()](settings_signal);
            let src = &src_transpose
                .into_iter()
                .map(|v| v[..v.len() - 1].to_vec())
                .collect::<Vec<Vec<f64>>>();
            (
                signal_name.as_str(),
                (
                    signal.bf(
                        &get_in_from_settings(
                            &settings_signal.used_ind,
                            &settings_signal.used_src,
                            &settings_signal.procedure_used_src,
                            settings_indicators,
                            src,
                            map_indicators,
                        ),
                        &get_signals_arg_from_settings(
                            &settings_signal.used_signals,
                            &settings_signal.procedure_used_signals,
                            settings_signals,
                            settings_indicators,
                            src,
                            map_signals,
                            map_indicators,
                        ),
                    ),
                    signal,
                ),
            )
        })
        .collect()
}

#[derive(Default)]
pub struct SignalsReady<'a> {
    pub signals_ready_without_bf: MAP<&'a str, Box<dyn SignalReady>>,
    pub signals_ready: MAP<&'a str, (BF_SIGNALS<'a>, Box<dyn SignalReady>)>,
}

impl<'a> SignalsReady<'a> {
    pub fn new(
        s_signals_ready: &'a SETTINGS_SIGNALS,
        s_indicators: &'a SETTINGS_INDS,
        funcs_extract_args: &MAP<&'a str, fn(&SETTINGS_SIGNAL) -> Box<dyn SignalReady>>,
        src_transpose: &[Vec<f64>],
        map_indicators: &MAP<&'a str, Box<dyn Indicator>>,
    ) -> Self {
        let signals_ready_without_bf =
            get_signals_from_settings_without_bf(s_signals_ready, funcs_extract_args);
        Self {
            signals_ready: get_signals_from_settings(
                s_signals_ready,
                s_indicators,
                funcs_extract_args,
                src_transpose,
                &signals_ready_without_bf,
                map_indicators,
            ),
            signals_ready_without_bf: signals_ready_without_bf,
        }
    }
    pub fn update_bf<'b>(
        &mut self,
        src_transpose: &[Vec<f64>],
        s: &'a SETTINGS,
        fa: &'b FUNCS_EXTRACT_ARGS_TYPE<SETTINGS_SIGNAL, Box<dyn SignalReady>>,
        indicators_without_bf: &MAP<&'a str, Box<dyn Indicator>>,
    ) {
        self.signals_ready = get_signals_from_settings(
            &s.signals_ready,
            &s.indications,
            fa,
            src_transpose,
            &self.signals_ready_without_bf,
            indicators_without_bf,
        );
    }
}

#[derive(Default)]
pub struct SignalsReadyGateway<'a> {
    pub signals_ready: *const SignalsReady<'a>,
    pub indicators: *const Indicators<'a>,
    pub settings_signals: *const SETTINGS_SIGNALS,
    pub settings_indicators: *const SETTINGS_INDS,
}

impl<'a> SignalsReadyGateway<'a> {
    pub fn new(
        signals_ready: *const SignalsReady<'a>,
        indicators: *const Indicators<'a>,
        settings_signals: *const SETTINGS_SIGNALS,
        settings_indicators: *const SETTINGS_INDS,
    ) -> Self {
        Self {
            signals_ready,
            indicators,
            settings_signals,
            settings_indicators,
        }
    }
    pub fn signals_series(
        &self,
        indications: &MAP<&'a str, f64>,
        src_transpose: &[Vec<f64>],
    ) -> MAP<&'a str, Signal> {
        unsafe { &*self.settings_signals }
            .iter()
            .fold(MAP::default(), |mut map, setting| {
                let key_uniq_str = setting.0.as_str();
                let mut src_arg = vec![];
                let mut signals_arg = vec![];
                for src_arg_el in &setting.1.used_src {
                    src_arg.push({
                        let sk = &src_transpose[src_arg_el.index];
                        sk[sk.len() - 1 - src_arg_el.sub_from_last_i]
                    });
                }
                for ind_arg_el in &setting.1.used_ind {
                    src_arg.push(indications[ind_arg_el.as_str()]);
                }
                for signals_arg_el in &setting.1.used_signals {
                    signals_arg.push(map[signals_arg_el.as_str()].clone());
                }
                if !setting.1.procedure_used_src.is_empty() {
                    src_arg = setting
                        .1
                        .procedure_used_src
                        .iter()
                        .map(|i| src_arg[*i])
                        .collect();
                }
                if !setting.1.procedure_used_signals.is_empty() {
                    src_arg = setting
                        .1
                        .procedure_used_signals
                        .iter()
                        .map(|i| src_arg[*i])
                        .collect();
                }
                let signal = unsafe { &(&(*self.signals_ready).signals_ready)[key_uniq_str] };
                map.insert(
                    key_uniq_str,
                    signal
                        .1
                        .signal_with_bf(&src_arg, &signals_arg, &signal.0, 0),
                );
                map
            })
    }
    pub fn signals_vec(
        &self,
        src_transpose: &[Vec<f64>],
    ) -> MAP<&'a str, Vec<Signal>> {
        unsafe { &*self.settings_signals }
            .iter()
            .map(|(k, setting)| {
                let key_uniq = k.as_str();
                let signal = unsafe { &(&(*self.signals_ready).signals_ready)[key_uniq] };
                (
                    key_uniq,
                    signal.1.signals_vec(
                        &get_in_from_settings(
                            &setting.used_ind,
                            &setting.used_src,
                            &setting.procedure_used_src,
                            unsafe { &*self.settings_indicators },
                            src_transpose,
                            unsafe { &(*self.indicators).indicators_without_bf },
                        ),
                        &get_signals_arg_from_settings(
                            &setting.used_signals,
                            &setting.procedure_used_signals,
                            unsafe { &*self.settings_signals },
                            unsafe { &*self.settings_indicators },
                            src_transpose,
                            unsafe { &(*self.signals_ready).signals_ready_without_bf },
                            unsafe { &(*self.indicators).indicators_without_bf },
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
    use bc_pack_indicators::FUNCS_EXTRACT_ARGS as FUNCS_EXTRACT_ARGS_IND;
    use bc_pack_signals_ready::FUNCS_EXTRACT_ARGS as FUNCS_EXTRACT_ARGS_SR;
    use bc_signals::ready::{
        change_signal::CHANGE_SIGNAL, convert::CONVERT, invert::INVERT, th::TH,
    };
    use bc_utils_lg::statics::prices::{SRC, SRC_TRANSPOSE};
    use bc_utils_lg::structs::settings::{
        SETTINGS_IND, SETTINGS_INDS, SETTINGS_SIGNAL, SETTINGS_SIGNALS, SETTINGS_USED_SRC,
    };
    use bc_utils_lg::types::maps::MAP;
    use pretty_assertions::assert_eq as assert_eq_pr;

    use bc_indicators_gw::gw::*;

    #[test]
    fn signals_from_settings_without_bf_res_1() {
        let settings = SETTINGS_SIGNALS::from_iter([(
            "th_1".to_string(),
            SETTINGS_SIGNAL { key: "th".to_string(), ..Default::default() },
        )]);
        let funcs_extract_args = FUNCS_EXTRACT_ARGS_SR();
        let res = get_signals_from_settings_without_bf(&settings, &funcs_extract_args);
        let res_1 = res.get("th_1").unwrap().as_ref();
        let rsi_test_1 = TH::default();
        let rsi_test_2 = (res_1 as &dyn Any).downcast_ref::<TH>().unwrap();
        assert_eq_pr!(&rsi_test_1, rsi_test_2);
    }

    #[test]
    fn signals_ready_res_1() {
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
        let signals = SignalsReady::new(
            &settings_signals,
            &settings_indicators,
            &FUNCS_EXTRACT_ARGS_SR(),
            &SRC_TRANSPOSE,
            &indicators.indicators_without_bf,
        );
        let indicators_gw = IndicatorsGateway::new(&indicators, &settings_indicators);
        let indications = indicators_gw.indications_series(&SRC_TRANSPOSE);
        let signals_gw = SignalsReadyGateway::new(
            &signals,
            &indicators,
            &settings_signals,
            &settings_indicators,
        );
        let res_1 = signals_gw.signals_series(&indications, &SRC_TRANSPOSE)["invert_1"];
        let res_2 = INVERT::default().signal(
            &vec![],
            &vec![vec![
                CHANGE_SIGNAL::default().signal(
                    &vec![],
                    &CONVERT::default()
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
    fn signals_ready_vec_res_1() {
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
        let res_1 = &signals_gw.signals_vec(&SRC_TRANSPOSE)["invert_1"];
        let res_2 = &INVERT::default().signals_vec(
            &vec![],
            &CHANGE_SIGNAL::default()
                .signals_vec(
                    &vec![],
                    &CONVERT::default()
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
