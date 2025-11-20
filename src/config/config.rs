use serde::Deserialize;
use paste::paste;

/// Define configuration sections and their corresponding "raw" (partially specified)
/// versions, and generate merging logic between them.
///
/// # Example
/// ```rust
/// config! {
///     section Keymap {
///         delete: String,
///     }
/// }
/// ```
///
/// This will generate:
///
/// ```rust
/// #[derive(Deserialize)]
/// pub struct Config {
///     pub keymap: Keymap,
/// }
///
/// #[derive(Deserialize)]
/// pub struct Keymap {
///    pub delete: String,
/// }
///
/// #[derive(Deserialize)]
/// pub struct RawConfig {}
///
/// impl Config {
///    pub fn merge(self, raw: RawConfig) -> Self { ... }
/// }
macro_rules! config {
    (
        $(
            section $section_name:ident {
                $($field_name:ident : $field_type:ty),* $(,)?
            }
        ),* $(,)?
    ) => {
        paste! {
            $(
                #[derive(Deserialize)]
                pub struct $section_name {
                    $(pub $field_name: $field_type),*
                }

                #[derive(Deserialize)]
                struct [<Raw $section_name>] {
                    $($field_name: Option<$field_type>),*
                }

                impl $section_name {
                    fn merge(self, raw: [<Raw $section_name>]) -> Self {
                        Self {
                            $(
                                $field_name: raw.$field_name.unwrap_or(self.$field_name)
                            ),*
                        }
                    }
                }
            )*


            #[derive(Deserialize)]
            pub struct Config {
                $(pub [<$section_name:snake>]: $section_name),*
            }

            #[derive(Deserialize)]
            pub struct RawConfig {
                $([<$section_name:snake>]: Option<[<Raw $section_name>]>),*
            }

            impl Config {
                pub fn merge(self, raw: RawConfig) -> Self {
                    Self {
                        $(
                            [<$section_name:snake>]: match raw.[<$section_name:snake>] {
                                Some(r) => self.[<$section_name:snake>].merge(r),
                                None => self.[<$section_name:snake>],
                            }
                        ),*
                    }
                }
            }
        }
    };
}

config! {
    section Keymap {
        delete: String,
        interact: String
    }
}
