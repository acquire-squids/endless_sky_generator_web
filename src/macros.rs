#[macro_export]
macro_rules! __wasm_newtype {
    (
        $(using $($want:path $(, $wants:path)* $(,)? )? ;)?
        in $mod_name:ident =>
        $v:vis $name:ident as $original:ty ;
        $($fv:vis $field:ident : $field_ty:ty $(=> $field_map:expr)?,)+
    ) => {
        #[cfg(all(target_family = "wasm", target_os = "unknown"))]
        $v mod $mod_name {
            $($(
                use $want;
                $(
                    use $wants;
                )*
            )?)?

            use wasm_bindgen::prelude::*;

            impl From<$name> for $original {
                fn from(value: $name) -> Self {
                    Self::new($($($field_map)? (value.$field),)+)
                }
            }

            #[wasm_bindgen]
            #[derive(Debug)]
            $v struct $name {
                $($fv $field: $field_ty,)+
            }

            #[wasm_bindgen]
            impl $name {
                #[wasm_bindgen(constructor)]
                #[allow(clippy::missing_const_for_fn)]
                #[must_use]
                $v fn new($($field: $field_ty,)+) -> Self {
                    Self { $($field,)+ }
                }
            }
        }

        $v use $mod_name::$name;
    };

    (
        $(using $($want:path $(, $wants:path)* $(,)? )? ;)?
        in $mod_name:ident =>
        $v:vis $name:ident ;
        $($fv:vis $field:ident : $field_ty:ty $(=> $field_map:expr)?,)+
    ) => {
        $v mod $mod_name {
            $($(
                use $want;
                $(
                    use $wants;
                )*
            )?)?

            #[derive(Debug, Clone)]
            $v struct $name {
                $($fv $field: $field_ty,)+
            }

            impl $name {
                #[must_use]
                $v fn new($($field: $field_ty,)+) -> Self {
                    Self { $($field $(: $field_map($field))?,)+ }
                }

                $(
                    #[must_use]
                    $v const fn $field(&self) -> &$field_ty {
                        &self.$field
                    }
                )+
            }
        }

        $v use $mod_name::$name;
    };
}

pub use __wasm_newtype as wasm_newtype;
