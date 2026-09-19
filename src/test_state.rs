use bc_utils_lg::prelude::*;

pub static SIGNALS_STATE: LazyLock<MAP<&str, Signal>> = LazyLock::new(|| {
    MAP::from_iter([
        ("th_1", Signal::new(0., 1.)),
        ("invert_1", Signal::new(0., 1.)),
        ("signal", Signal::new(1., 1.)),
    ])
});
