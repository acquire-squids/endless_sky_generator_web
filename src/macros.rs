#[macro_export]
macro_rules! __wasm_newtype {
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

            #[cfg(all(target_family = "wasm", target_os = "unknown"))]
            use wasm_bindgen::prelude::*;

            #[cfg_attr(all(target_family = "wasm", target_os = "unknown"), wasm_bindgen)]
            #[derive(Debug)]
            $v struct $name {
                $($fv $field: $field_ty,)+
            }

            #[cfg_attr(all(target_family = "wasm", target_os = "unknown"), wasm_bindgen)]
            impl $name {
                #[cfg_attr(all(target_family = "wasm", target_os = "unknown"), wasm_bindgen(constructor))]
                #[must_use]
                $v fn new($($field: $field_ty,)+) -> Self {
                    Self { $($field,)+ }
                }
            }

            impl $name {
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
