/// Languages any site in the family may use. Each site picks a subset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Lang {
    En,
    De,
    Fr,
    It,
}

impl Lang {
    pub const ALL: [Lang; 4] = [Lang::En, Lang::De, Lang::Fr, Lang::It];

    /// BCP 47 code, also the URL prefix for non-default languages.
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::De => "de",
            Lang::Fr => "fr",
            Lang::It => "it",
        }
    }

    /// Open Graph locale.
    pub fn locale(self) -> &'static str {
        match self {
            Lang::En => "en_GB",
            Lang::De => "de_CH",
            Lang::Fr => "fr_CH",
            Lang::It => "it_CH",
        }
    }

    /// Name of the language in that language, for switch links.
    pub fn native_name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::De => "Deutsch",
            Lang::Fr => "Français",
            Lang::It => "Italiano",
        }
    }

    pub fn from_code(code: &str) -> Option<Lang> {
        Lang::ALL.into_iter().find(|l| l.code() == code)
    }
}
