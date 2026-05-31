use bitflags::bitflags;

bitflags! {
    #[derive(PartialEq, Eq)]
    pub struct PatchFlags: i16 {
        const TEXT = 1;
        const CLASS = 1 << 1;
        const STYLE = 1 << 2;
        const PROPS = 1 << 3;
        const FULL_PROPS = 1 << 4;
        const HYDRATE_EVENTS = 1 << 5;
        const STABLE_FRAGMENT = 1 << 6;
        const KEYED_FRAGMENT = 1 << 7;
        const UNKEYED_FRAGMENT = 1 << 8;
        const NEED_PATCH = 1 << 9;
        const DYNAMIC_SLOTS = 1 << 10;
        const HOISTED = -1;
        const BAIL = -2;
    }
}
