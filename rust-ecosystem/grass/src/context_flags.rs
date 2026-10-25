use std::ops::{BitAnd, BitOr, BitOrAssign};

#[derive(Debug, Copy, Clone)]
pub(crate) struct ContextFlags(pub u16);

#[derive(Debug, Copy, Clone)]
pub(crate) struct ContextFlag(u16);

impl ContextFlags {
    pub const IN_MIXIN: ContextFlag = ContextFlag(1);
    pub const IN_FUNCTION: ContextFlag = ContextFlag(1 << 1);
    pub const IN_CONTROL_FLOW: ContextFlag = ContextFlag(1 << 2);
    pub const IN_KEYFRAMES: ContextFlag = ContextFlag(1 << 3);
    pub const FOUND_CONTENT_RULE: ContextFlag = ContextFlag(1 << 4);
    pub const IN_STYLE_RULE: ContextFlag = ContextFlag(1 << 5);
    pub const IN_UNKNOWN_AT_RULE: ContextFlag = ContextFlag(1 << 6);
    pub const IN_CONTENT_BLOCK: ContextFlag = ContextFlag(1 << 7);
    pub const IS_USE_ALLOWED: ContextFlag = ContextFlag(1 << 8);
    pub const IN_PARENS: ContextFlag = ContextFlag(1 << 9);
    pub const AT_ROOT_EXCLUDING_STYLE_RULE: ContextFlag = ContextFlag(1 << 10);
    pub const IN_SUPPORTS_DECLARATION: ContextFlag = ContextFlag(1 << 11);
    pub const IN_SEMI_GLOBAL_SCOPE: ContextFlag = ContextFlag(1 << 12);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub fn unset(&mut self, flag: ContextFlag) {
        self.0 &= !flag.0;
    }

    pub fn set(&mut self, flag: ContextFlag, v: bool) {
        if v {
            self.0 |= flag.0;
        } else {
            self.unset(flag);
        }
    }

    pub fn in_mixin(self) -> bool {
        (self.0 & Self::IN_MIXIN) != 0
    }

    pub fn in_function(self) -> bool {
        (self.0 & Self::IN_FUNCTION) != 0
    }

    pub fn in_control_flow(self) -> bool {
        (self.0 & Self::IN_CONTROL_FLOW) != 0
    }

    pub fn in_keyframes(self) -> bool {
        (self.0 & Self::IN_KEYFRAMES) != 0
    }

    pub fn in_style_rule(self) -> bool {
        (self.0 & Self::IN_STYLE_RULE) != 0
    }

    pub fn in_unknown_at_rule(self) -> bool {
        (self.0 & Self::IN_UNKNOWN_AT_RULE) != 0
    }

    pub fn in_content_block(self) -> bool {
        (self.0 & Self::IN_CONTENT_BLOCK) != 0
    }

    pub fn in_parens(self) -> bool {
        (self.0 & Self::IN_PARENS) != 0
    }

    pub fn at_root_excluding_style_rule(self) -> bool {
        (self.0 & Self::AT_ROOT_EXCLUDING_STYLE_RULE) != 0
    }

    pub fn in_supports_declaration(self) -> bool {
        (self.0 & Self::IN_SUPPORTS_DECLARATION) != 0
    }

    pub fn in_semi_global_scope(self) -> bool {
        (self.0 & Self::IN_SEMI_GLOBAL_SCOPE) != 0
    }

    pub fn found_content_rule(self) -> bool {
        (self.0 & Self::FOUND_CONTENT_RULE) != 0
    }

    pub fn is_use_allowed(self) -> bool {
        (self.0 & Self::IS_USE_ALLOWED) != 0
    }
}

impl BitAnd<ContextFlag> for u16 {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: ContextFlag) -> Self::Output {
        self & rhs.0
    }
}

impl BitOr<ContextFlag> for ContextFlags {
    type Output = Self;
    fn bitor(self, rhs: ContextFlag) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign<ContextFlag> for ContextFlags {
    fn bitor_assign(&mut self, rhs: ContextFlag) {
        self.0 |= rhs.0;
    }
}
