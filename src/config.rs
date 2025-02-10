use std::{io::Write, path::PathBuf};

use directories::ProjectDirs;
use iced::Theme;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use smart_default::SmartDefault;
use snafu::{OptionExt, ResultExt, Whatever};

#[derive(SmartDefault, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    #[serde(serialize_with = "ser_theme", deserialize_with = "de_theme")]
    #[default(Theme::CatppuccinMocha)]
    pub theme: Theme,
}

impl Config {
    pub fn open(path: Option<PathBuf>) -> Result<Self, Whatever> {
        let path = match path {
            Some(path) => path,
            None => {
                let proj_dirs = ProjectDirs::from("com", "tukanoidd", "waypwr")
                    .whatever_context("Unable to get project directories")?;

                let config_dir = proj_dirs.config_local_dir();

                if !config_dir.exists() {
                    tracing::warn!(
                        "Config directory at {config_dir:?} doesn't exist, creating a new one"
                    );

                    std::fs::create_dir_all(config_dir).whatever_context(format!(
                        "Failed to create a config directory at {config_dir:?}"
                    ))?;
                }

                config_dir.join("config.toml")
            }
        };

        match path.exists() {
            true => toml::from_str(
                &std::fs::read_to_string(&path).whatever_context("Failed to read config")?,
            )
            .whatever_context("Failed to parse config"),

            false => {
                tracing::warn!("No config file found, creating a default one at {path:?}");

                let config = Self::default();

                let mut file = std::fs::File::create(&path).whatever_context(format!(
                    "Failed to create a default config file at {path:?}"
                ))?;
                file.write_all(
                    toml::to_string_pretty(&config)
                        .whatever_context("Failed to serialize default config")?
                        .as_bytes(),
                )
                .whatever_context(format!(
                    "Failed to write a default config to a file at {path:?}"
                ))?;

                Ok(config)
            }
        }
    }
}

macro_rules! serde_theme {
    (
        $ty:ty => [
            $($name:ident),+
            $(,)?
        ]
    ) => {
        pub fn parse_theme_str(str: &str) -> Result<$ty, String> {
            use heck::ToKebabCase;

            $(
                if str == &stringify!($name).to_kebab_case() {
                    return Ok(<$ty>::$name);
                }
            )+

            Err(format!("Failed to parse theme name from string: {str}"))
        }

        fn ser_theme<S>(val: &$ty, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            use heck::ToKebabCase;

            let str = match val {
                $(<$ty>::$name => stringify!($name).to_kebab_case(),)+
                _ => return Err(serde::ser::Error::custom("Custom themes are not supported!"))
            };

            serializer.serialize_str(&str)
        }

        fn de_theme<'de, D>(deserializer: D) -> Result<Theme, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_str(ThemeVisitor)
        }

        struct ThemeVisitor;

        impl Visitor<'_> for ThemeVisitor {
            type Value = Theme;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "A string name of the theme")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use heck::ToKebabCase;

                $(
                    if v == &stringify!($name).to_kebab_case() {
                        return Ok(<$ty>::$name);
                    }
                )+

                Err(serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(v),
                    &format!(
                        "{:?}",
                        [$(stringify!($name).to_kebab_case()),+]
                    ).as_str()
                ))
            }
        }
    }
}

serde_theme!(Theme => [
    Light,
    Dark,
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
    GruvboxLight,
    GruvboxDark,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    KanagawaWave,
    KanagawaDragon,
    KanagawaLotus,
    Moonfly,
    Nightfly,
    Oxocarbon,
    Ferra
]);
